//! Содержит определение подготовленных выражений, которые используются для эффективного исполнения запросов,
//! и структур, связанных с ними.
mod storage;

use std::mem;
use std::os::raw::{c_int, c_short, c_void, c_uchar, c_uint, c_ushort};
use std::ptr;

use {Connection, DbResult, Result};
use error::{self, Error};
use error::DbError::{Info, NoData, Fault};
use types::{FromDB, Type, Syntax};

use ffi::{Descriptor, Handle};// Основные типобезопасные примитивы
use ffi::ParamHandle;// Типажи для безопасного моста к FFI

use ffi::attr::AttrHolder;
use ffi::native::{OCIParam, OCIStmt, OCIBind, OCIError};// FFI типы
use ffi::native::{OCIParamGet, OCIStmtExecute, OCIStmtRelease, OCIStmtPrepare2, OCIStmtFetch2, OCIBindByPos, OCIBindByName, OCIDefineByPos};// FFI функции
use ffi::types::Attr;
use ffi::types::{DefineMode, CachingMode, ExecuteMode, FetchMode};

use self::storage::DefineInfo;

//-------------------------------------------------------------------------------------------------
fn param_get<'d, T: ParamHandle>(handle: *const T, pos: c_uint, err: &Handle<OCIError>) -> DbResult<Descriptor<'d, OCIParam>> {
  let mut desc = ptr::null_mut();
  let res = unsafe {
    OCIParamGet(
      handle as *const c_void, T::ID as c_uint,
      err.native_mut(),
      &mut desc, pos
    )
  };
  Descriptor::from_ptr(res, desc as *const OCIParam, err)
}

//-------------------------------------------------------------------------------------------------
/// Структура для представления колонки базы данных из списка выбора
#[derive(Debug)]
pub struct Column {
  /// Порядковый номер колонки в списке выбора (нумерация с 0)
  pub pos: usize,
  /// Тип колонки в базе данных.
  pub type_: Type,
  /// Название колонки в списке выбора (т.е. либо название колонки в базе данных, либо ее псевдоним).
  pub name: String,
  /// Ширина колонки в байтах. Показывает, сколько байт максимум может занимать значение колонки,
  /// а не занимаемый реально данными объем.
  pub size: usize,
  /// Количество десятичных цифр для представления чисел для числовых данных.
  /// Количество десятичных цифр, отводимых под год/день для интервальных типов.
  pub precision: usize,
  /// Returns the scale (number of digits to the right of the decimal point) for conversions from packed and zoned decimal input data types.
  pub scale: usize,
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_: c_ushort = try!(desc.get_(Attr::DataType, err));
    let name  = try!(desc.get_str(Attr::Name, err));
    //let ischar= try!(desc.get_(Attr::CharUsed, err));
    //let size : c_uint  = try!(desc.get_(Attr::CharSize, err));
    let size : c_uint = try!(desc.get_(Attr::DataSize, err));
    let prec : c_uint = try!(desc.get_(Attr::Precision, err));
    //FIXME: Атрибуты Server и Scale имеют одинаковое представление в C-коде (6), но в Rust-е наличие перечислений с одним значением
    // запрещено.
    // let scale: c_uint = try!(desc.get_(Attr::Scale, err));
    let scale: c_uint = try!(desc.get_(Attr::Server, err));

    Ok(Column {
      pos: pos,
      name: name,
      size: size as usize,
      type_: unsafe { mem::transmute(type_ as u16) },
      precision: prec as usize,
      scale: scale as usize,
    })
  }
  /// Для биндинга значений через `OCIBindByPos`, `OCIBindByName` и `OCIDefineByPos` для некоторых типов
  /// столбцов необходимо передавать не тот тип, что в столбце записан, а другой, в частности, вместо
  ///  `SQLT_NUM` требуется передавать `SQLT_VNU`.
  fn bind_type(&self) -> Type {
    match self.type_ {
      Type::NUM => Type::VNU,
      t => t,
    }
  }
}
//-------------------------------------------------------------------------------------------------
/// Подготовленное выражение.
#[derive(Debug)]
pub struct Statement<'conn, 'key> {
  /// Соединение, которое подготовило данное выражение
  conn: &'conn Connection<'conn>,
  /// Внутренний указатель оракла на подготовленное выражение
  native: *const OCIStmt,
  /// Ключ для кеширования выражения
  key: Option<&'key str>,
}
impl<'conn, 'key> Statement<'conn, 'key> {
  /// Получает хендл для записи ошибок во время общения с базой данных. Хендл берется из соединения, которое породило
  /// данное выражение. В случае возникновения ошибки при вызове FFI-функции она может быть получена из хендла с помощью
  /// вызова `decode(ffi_result)`.
  #[inline]
  fn error(&self) -> &Handle<OCIError> {
    self.conn.error()
  }
  /// # Параметры
  /// - count:
  ///   * Для `select` выражений это количество строк, которые нужно извлечь prefetch-ем, уже в момент выполнения
  ///     запроса (т.е. сервер БД вернет их, не дожидаясь вызова `OCIStmtFetch2`). Если prefetch не нужен, то должно
  ///     быть равно `0`.
  ///   * Для не-`select` выражений это номер последнего элемента в буфере данных со связанными параметрами, которые
  ///     нужно использовать при выполнении данной операции
  /// - offset:
  ///   Смещение с буфере со связанными переменными, с которого необходимо начать выполнение 
  fn execute_impl(&self, count: c_uint, offset: c_uint, mode: ExecuteMode) -> DbResult<()> {
    let res = unsafe {
      OCIStmtExecute(
        self.conn.context.native_mut(),
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        offset,
        ptr::null(),
        ptr::null_mut(),
        mode as c_uint
      )
    };
    return self.error().check(res);
  }
  /// Извлекает из текущего выражения данные, которые в нем имеются после выполнения `select`-а.
  ///
  /// # Параметры
  /// - count:
  ///   Количество строк, которые нужно получить из текущей позиции курсора
  /// - index:
  ///   Для режимов `Absolute` и `Relative` определяет номер извлекаемого элемента, в остальных случаях игнорируется.
  fn fetch(&self, count: c_uint, mode: FetchMode, index: c_int) -> DbResult<()> {
    let res = unsafe {
      OCIStmtFetch2(
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        mode as c_ushort,
        index as c_int,
        0 // Неясно, что такое
      )
    };
    return self.error().check(res);
  }
  fn bind_by_pos(&self, pos: c_uint, value: *mut c_void, size: c_int, dty: Type) -> DbResult<Handle<OCIBind>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIBindByPos(
        self.native as *mut OCIStmt,
        &mut handle,
        self.error().native_mut(),
        pos,
        // Указатель на данные для извлечения результата, его размер и тип
        value, size, dty as c_ushort,
        ptr::null_mut(),// Массив индикаторов (null/не null, пока не используем)
        ptr::null_mut(),// Массив длин для каждого значения
        ptr::null_mut(),// Массив для column-level return codes

        0, ptr::null_mut(), 0
      )
    };

    Handle::from_ptr(res, handle, self.error().native_mut())
  }
  fn bind_by_name(&self, placeholder: &str, value: *mut c_void, size: c_int, dty: Type) -> DbResult<Handle<OCIBind>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIBindByName(
        self.native as *mut OCIStmt,
        &mut handle,
        self.error().native_mut(),
        placeholder.as_ptr() as *const c_uchar, placeholder.len() as c_int,
        // Указатель на данные для извлечения результата, его размер и тип
        value, size, dty as c_ushort,
        ptr::null_mut(),// Массив индикаторов (null/не null, пока не используем)
        ptr::null_mut(),// Массив длин для каждого значения
        ptr::null_mut(),// Массив для column-level return codes

        0, ptr::null_mut(), 0
      )
    };

    Handle::from_ptr(res, handle, self.error().native_mut())
  }
  /// Ассоциирует с выражением адреса буферов, в которые извлечь данные.
  ///
  /// # Параметры
  /// - pos:
  ///   Порядковый номер параметра в запросе (нумерация с 0)
  /// - dty:
  ///   Тип данных, которые нужно извлечь
  /// - buf:
  ///   Буфер, в который будет записана выходная информация.
  /// - ind:
  ///   Переменная, в которую будет записан признак того, что в столбце содержится `NULL`.
  /// - out_size:
  ///   Количество байт, записанное в буфер. Не превышает его длину
  fn define(&self, pos: usize, dty: Type, buf: &mut DefineInfo, mode: DefineMode) -> DbResult<()> {
    let res = unsafe {
      OCIDefineByPos(
        self.native as *mut OCIStmt,
        //TODO: Чтобы управлять временем жизни хендла, нужно передать корректный хендл, но тогда его придется
        // самому закрывать. Пока нам это не нужно
        &mut ptr::null_mut(),
        self.error().native_mut(),
        // В API оракла нумерация с 1, мы же придерживаемся традиционной с 0
        (pos + 1) as c_uint,
        // Указатель на данные для размещения результата, его размер и тип
        buf.as_ptr(), buf.capacity(), dty as c_ushort,
        &mut buf.is_null as *mut c_short as *mut c_void,// Массив индикаторов (null/не null)
        buf.size_mut(),// Массив длин для каждого значения, которое извлекли из базы
        &mut buf.ret_code as *mut c_ushort,// Массив для column-level return codes
        mode as c_uint
      )
    };
    self.error().check(res)
  }
  /// Получает количество столбцов, извлеченный в `SELECT`-выражении. Необходимо вызывать после выполнения `SELECT`-запроса,
  /// т.к. до этого момента? или в случае выполнения не `SELECT`-запроса, эта информация недоступна.
  #[inline]
  fn param_count(&self) -> DbResult<c_uint> {
    self.get_(Attr::ParamCount, self.error())
  }
  /// Получает количество cтрок, обработанных последним выполненным `INSERT/UPDATE/DELETE` запросом,
  /// или количество строк, полученное последним вызовом fetch для `SELECT` запроса.
  #[inline]
  fn row_count(&self) -> DbResult<c_uint> {
    self.get_(Attr::RowCount, self.error())
  }
  /// Получает дескриптор с описанием столбца в полученном списке извлеченных `SELECT`-ом столбцов для указанного столбца.
  ///
  /// # Параметры
  /// - pos:
  ///   Номер столбца, для которого извлекается информация. Нумерация с 0, в отличие от API оракла, где нумерация идет с 1.
  #[inline]
  fn param_get(&self, pos: c_uint) -> DbResult<Descriptor<OCIParam>> {
    param_get(self.native, pos + 1, self.error())
  }

  /// Получает информацию о списке выбора `SELECT`-выражения.
  fn columns(&self) -> Result<Vec<Column>> {
    let cnt = try!(self.param_count());
    let mut vec = Vec::with_capacity(cnt as usize);
    for i in 0..cnt {
      vec.push(try!(Column::new(i as usize, try!(self.param_get(i)), self.error())));
    }
    Ok(vec)
  }
  /// Возвращает соединение, из которого было подготовлено данные выражение.
  #[inline]
  pub fn connection(&self) -> &Connection {
    self.conn
  }
  /// Выполняет `SELECT`-запрос и возвращает ленивый итератор по результатам. Результаты будут извлекаться из итератора только по мере
  /// его продвижения. Так как неизвлеченные данные при этом на сервере хранятся в буферах, привязанных к конкретному выражению, то
  /// данный метод требует `mut`-ссылки. Таким образом, гарантируется, что никто не сможет выполнить новое выражение и затереть набор,
  /// пока по нему итерируются.
  ///
  /// Таким образом, чтобы выполнить следующий запрос над тем же самым выражением, старый результат вызова данной функции должен покинуть
  /// область видимости и освободить таким образом `mut`-ссылку.
  ///
  /// Обратите внимание, что выполнение не-`SELECT` запросов через вызов [`execute()`][1] не требует изменяемой ссылки, т.к. в этом случае
  /// все данные результата возвращаются сервером сразу.
  ///
  /// # Пример
  /// ```
  /// # use oci::Environment;
  /// # use oci::params::{ConnectParams, Credentials};
  /// # let env = Environment::new(Default::default()).unwrap();
  /// # let conn = env.connect(ConnectParams { dblink: "".into(), attach_mode: Default::default(), credentials: Credentials::Ext, auth_mode: Default::default() }).unwrap();
  /// let mut stmt = conn.prepare("select * from user_users").unwrap();
  /// {
  ///   // Используем анонимный блок, чтобы можно было выполнить присваивание в rs2 ниже, когда заимствование
  ///   // stmt как изменяемой ссылки для rs закончится.
  ///   let rs = stmt.query().unwrap();
  ///   for row in &rs {
  ///     let user: Option<String> = row.get(0).unwrap();
  ///     println!("user: {:?}", user);
  ///   }
  /// }
  /// let rs2 = stmt.query().unwrap();
  /// // ...продолжение работы...
  /// ```
  ///
  /// # OCI вызовы
  /// Для выполнения выражения непосредственно при вызове данной функции используется OCI-вызов [`OCIStmtExecute()`][2]. Для последующего
  /// извлечения данных через итератор используется вызов [`OCIStmtFetch2()`][3], один на каждую итерацию (данное поведение будет улучшено
  /// в дальнейшем, для получения результатов порциями некоторого настраиваемого размера).
  ///
  /// # Запросы к серверу (1..)
  /// Непосредственно в момент вызова данной функции выполняется один вызов [`OCIStmtExecute()`][2]. Каждая итерация выполняет по одному
  /// вызову [`OCIStmtFetch2()`][3].
  ///
  /// [1]: #method.execute
  /// [2]: https://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17163
  /// [3]: https://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17165
  pub fn query(&mut self) -> Result<RowSet> {
    try!(self.execute_impl(0, 0, Default::default()));

    RowSet::new(self)
  }
  /// Выполняет DDL или `INSERT/UPDATE/DELETE` запрос. В последнем случае возвращает количество строк,
  /// затронутых запросом (т.е. количество добавленных/обновленных/удаленных строк). Для DDL выражений
  /// (например, `create table`) возвращает `0`.
  ///
  /// # OCI вызовы
  /// Для выполнения выражения непосредственно при вызове данной функции используется OCI-вызов [`OCIStmtExecute()`][1]. Для последующего
  /// получения количества затронутых строк используется вызов [`OCIAttrGet()`][2].
  ///
  /// # Запросы к серверу (1)
  /// Непосредственно в момент вызова данной функции выполняется один вызов [`OCIStmtExecute()`][1].
  ///
  /// [1]: https://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17163
  /// [2]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17130
  pub fn execute(&self) -> Result<usize> {
    try!(self.execute_impl(1, 0, Default::default()));

    Ok(try!(self.row_count()) as usize)
  }
}
impl<'conn, 'key> Drop for Statement<'conn, 'key> {
  fn drop(&mut self) {
    let keyPtr = self.key.map_or(0 as *const c_uchar, |x| x.as_ptr() as *const c_uchar);
    let keyLen = self.key.map_or(0 as c_uint        , |x| x.len()  as c_uint);
    let res = unsafe { OCIStmtRelease(self.native as *mut OCIStmt, self.error().native_mut(), keyPtr, keyLen, 0) };
    self.error().check(res).expect("OCIStmtRelease");
  }
}
impl<'conn, 'key> AttrHolder<OCIStmt> for Statement<'conn, 'key> {
  fn holder_type() -> c_uint {
    ::ffi::types::Handle::Stmt as c_uint
  }
  fn native(&self) -> *const OCIStmt {
    self.native
  }
}

impl<'conn, 'key> super::StatementPrivate for Statement<'conn, 'key> {
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: Syntax) -> DbResult<Statement<'c, 'k>> {
    let mut stmt = ptr::null_mut();
    let keyPtr = key.map_or(0 as *const c_uchar, |x| x.as_ptr() as *const c_uchar);
    let keyLen = key.map_or(0 as c_uint        , |x| x.len()  as c_uint);
    let res = unsafe {
      OCIStmtPrepare2(
        conn.context.native_mut(),
        &mut stmt as *mut *mut OCIStmt,
        conn.error().native_mut(),
        // Текст SQL запроса
        sql.as_ptr() as *const c_uchar, sql.len() as c_uint,
        // Ключ кеширования, по которому достанется запрос, если он был закеширован
        keyPtr, keyLen,
        syntax as c_uint, CachingMode::Default as c_uint
      )
    };
    return match res {
      0 => Ok(Statement { conn: conn, native: stmt, key: key }),
      e => Err(conn.error().decode(e)),
    };
  }
}

/// Результат `SELECT`-выражения, представляющий одну строчку с данными из всей выборки.
#[derive(Debug)]
pub struct Row<'rs> {
  // Выборка, из которой получен данный объект
  rs: &'rs RowSet<'rs>,
  /// Массив данных для каждой колонки.
  data: Vec<DefineInfo<'rs>>,
  /// Диагностическая информация, полученная при извлечении данных, если есть.
  /// Например, может содержать информацию о том, что значение колонки было получено не полностью
  /// из-за недостаточного размера принимающего буфера.
  pub info: Option<Vec<error::Info>>,
}
impl<'d> Row<'d> {
  fn new(rs: &'d RowSet) -> Result<Self> {
    let mut data: Vec<DefineInfo> = Vec::with_capacity(rs.columns.len());

    for c in &rs.columns {
      data.push(try!(DefineInfo::new(rs.stmt, c)));
      // unwrap делать безопасно, т.к. мы только что вставили в массив данные
      try!(rs.stmt.define(c.pos, c.bind_type(), data.last_mut().unwrap(), Default::default()));
    }

    Ok(Row { rs: rs, data: data, info: None })
  }
  /// Получает описание столбца списка выбора результата `SELECT`-выражения по указанному индексу.
  #[inline]
  pub fn column<I: RowIndex>(&self, index: I) -> Result<&Column> {
    match index.idx(self.rs) {
      Some(idx) => Ok(&self.rs.columns()[idx]),
      None => Err(Error::InvalidColumn),
    }
  }
  /// Извлекает значение указанного типа из строки результата по заданному индексу.
  ///
  /// Возвращает ошибку в случае, если индекс некорректен или конвертация данных по указанному
  /// индексу невозможно в запрошенный тип, потому что его реализация типажа `FromDB` не поддерживает
  /// это.
  ///
  /// Если в столбце находится `NULL`, то возвращает `None`. Конвертация с помощью типажа `FromDB`
  /// выполняется только в том случае, если в базе находится не `NULL` значение.
  ///
  /// К сожалению, для типа `Row` невозможно реализовать типаж `Index`, чтобы использовать синтаксический сахар в виде
  /// получения результата индексацией квадратными скобками (`row[...]`). Это невозможно по двум причинам:
  ///
  /// 1. Компилятор не может вывести возвращаемый тип, т.к. типаж `Index` определяет его, как ассоциированный тип, а не
  ///    тип-параметр типажа. Таким образом, для возвращаемого типа невозможно указать ограничение на допустимые значения.
  /// 2. Даже если бы удалось победить первую проблему, типаж `Index` предусматривает возвращение ссылки на значение
  ///    вместо самого значения. Однако в случае реализации `get()` возвращаемое значение конструируется в момент получения,
  ///    таким образом, невозможно отдать ссылку на него, не сохранив предварительно внутри структуры `Row`
  pub fn get<T: FromDB, I: RowIndex>(&self, index: I) -> Result<Option<T>> {
    let col = try!(self.column(index));
    self.data[col.pos].to(col.bind_type(), self.rs.stmt.connection())
  }
}
/// Ленивый набор результатов, полученный при выполнении `SELECT` выражения. Реально данные извлекаются при итерации по набору,
/// именно поэтому метод [`query()`][1], возвращающий их, является `mut` методом.
///
/// В настоящий момент при итерации по набору получается одна строка за раз, с выполнением обращения к серверу.
///
/// [1]: ./struct.Statement.html#method.query
#[derive(Debug)]
pub struct RowSet<'stmt> {
  /// Выражение, выполнение которого дало данный набор результатов
  stmt: &'stmt Statement<'stmt, 'stmt>,
  /// Список колонок, которые извлекали из базы данных
  columns: Vec<Column>,
}
impl<'stmt> RowSet<'stmt> {
  /// Создает набор из выражения. Запоминает описание столбцов выражения
  #[inline]
  fn new(stmt: &'stmt Statement) -> Result<Self> {
    Ok(RowSet { stmt: stmt, columns: try!(stmt.columns()) })
  }
  /// Получает выражение, которое породило данный набор результатов.
  #[inline]
  pub fn statement(&self) -> &Statement<'stmt, 'stmt> {
    self.stmt
  }
  /// Получает список столбцов, которые содержатся в данном результате `SELECT`-а.
  #[inline]
  pub fn columns(&self) -> &[Column] {
    &self.columns
  }
  /// Продвигает итератор по текущему набору вперед, получает следующий элемент или `None`, если элементов больше не осталось.
  /// Поседение аналогично обычному итератору за тем исключением, что при ошибке извлечения данных возвращается `Err`, а не
  /// выполняется паника текущего потока.
  ///
  /// # OCI вызовы
  /// В настоящий момент при каждом вызове выполняется OCI-вызов [`OCIStmtFetch2()`][1], но это будет изменено в дальнейшем для выполнения
  /// пакетных чтений сразу же по несколько элементов (размер пакета будет конфигурируемым).
  ///
  /// # Запросы к серверу (1)
  /// Каждый вызов данной функции приводит к одному запросу к серверу.
  ///
  /// [1]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17165
  pub fn next(&'stmt self) -> Result<Option<Row<'stmt>>> {
    // Подготавливаем место в памяти для извлечения данных.
    // TODO: его можно переиспользовать, незачем создавать каждый раз.
    let mut r = Row::new(self).expect("Row::new failed");
    match self.stmt.fetch(1, Default::default(), 0) {
      Ok(_) => Ok(Some(r)),
      Err(Info(data)) => {
        r.info = Some(data);
        Ok(Some(r))
      }
      Err(NoData) => Ok(None),
      // ORA-01002: fetch out of sequence - если перезапустить итератор, из которого вычитаны все данные, вернется данная ошибка
      Err(Fault(error::Info { code: 1002, .. })) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }
}
impl<'stmt> Iterator for &'stmt RowSet<'stmt> {
  type Item = Row<'stmt>;

  fn next(&mut self) -> Option<Self::Item> {
    RowSet::next(self).expect("`fetch` failed")
  }
}

/// Типаж, позволяющий указать типы, которые можно использовать для индексации набора полей, полученных из базы данных,
/// для извлечения данных. Наиболее типичное применение -- использование индекса или имени колонки для извлечения данных.
/// Благодаря типажу для этого можно использовать одну и ту же функцию [`get()`][get].
///
/// [get]: ./struct.Row.html#method.get
pub trait RowIndex {
  /// Превращает объект в индекс, по которому можно извлечь данные, или в `None`, если нет индекса, соответствующего
  /// данному объекту. В этом случае при получении данных из столбца метод [`get()`][get] вернет ошибку [`InvalidColumn`][err].
  ///
  /// [get]: ./struct.Row.html#method.get
  /// [err]: ../error/enum.Error.html#variant.InvalidColumn
  fn idx(&self, rs: &RowSet) -> Option<usize>;
}

impl RowIndex for usize {
  fn idx(&self, rs: &RowSet) -> Option<usize> {
    if *self >= rs.columns().len() {
      return None;
    }
    Some(*self)
  }
}
impl<'a> RowIndex for &'a str {
  fn idx(&self, rs: &RowSet) -> Option<usize> {
    rs.columns().iter().position(|x| x.name == *self)
  }
}
