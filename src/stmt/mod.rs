//! Содержит определение подготовленных выражений, которые используются для эффективного исполнения запросов,
//! и структур, связанных с ними.
pub mod index;
pub mod query;
mod storage;

use std::mem;
use std::os::raw::c_void;
use std::ptr;

use {Connection, DbResult, Result};
use convert::BindInfo;
use types::{Type, Syntax, StatementType};

use ffi::{Descriptor, Handle};// Основные типобезопасные примитивы
use ffi::ParamHandle;// Типажи для безопасного моста к FFI

use ffi::attr::AttrHolder;
use ffi::native::{OCIBind, OCIParam, OCIStmt, OCIError};// FFI типы
use ffi::native::{OCIParamGet, OCIStmtExecute, OCIStmtRelease, OCIStmtPrepare2, OCIStmtFetch2, OCIBindByPos, OCIBindByName, OCIBindDynamic, OCIDefineByPos};// FFI функции
use ffi::native::bind::{InBindFn, in_bind_adapter};
use ffi::native::lob::LobPiece;
use ffi::types::Attr;
use ffi::types::{BindMode, DefineMode, CachingMode, ExecuteMode, FetchMode};

use self::index::BindIndex;
use self::storage::DefineInfo;
use self::query::RowSet;

//-------------------------------------------------------------------------------------------------
fn param_get<'d, T: ParamHandle>(handle: *const T, pos: u32, err: &Handle<OCIError>) -> DbResult<Descriptor<'d, OCIParam>> {
  let mut desc = ptr::null_mut();
  let res = unsafe {
    OCIParamGet(
      handle as *const c_void, T::ID as u32,
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
  pub size: u32,
  /// Количество десятичных цифр для представления чисел для числовых данных.
  /// Количество десятичных цифр, отводимых под год/день для интервальных типов.
  pub precision: u16,
  /// Returns the scale (number of digits to the right of the decimal point) for conversions from packed and zoned decimal input data types.
  pub scale: i8,
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_: u16 = try!(desc.get_(Attr::DataType, err));
    let name       = try!(desc.get_str(Attr::Name, err));
    //let ischar= try!(desc.get_(Attr::CharUsed, err));
    //let size = try!(desc.get_(Attr::CharSize, err));
    let size : u32 = try!(desc.get_(Attr::DataSize, err));
    let prec : u16 = try!(desc.get_(Attr::Precision, err));
    //FIXME: Атрибуты Server и Scale имеют одинаковое представление в C-коде (6), но в Rust-е наличие перечислений с одним значением
    // запрещено.
    // let scale: i8 = try!(desc.get_(Attr::Scale, err));
    let scale: i8 = try!(desc.get_(Attr::Server, err));

    Ok(Column {
      pos: pos,
      name: name,
      size: size,
      type_: unsafe { mem::transmute(type_) },
      precision: prec,
      scale: scale,
    })
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
  fn execute_impl(&self, count: u32, offset: u32, mode: ExecuteMode) -> DbResult<()> {
    let res = unsafe {
      OCIStmtExecute(
        self.conn.context.native_mut(),
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        offset,
        ptr::null(),
        ptr::null_mut(),
        mode as u32
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
  fn fetch(&self, count: u32, mode: FetchMode, index: i32) -> DbResult<()> {
    let res = unsafe {
      OCIStmtFetch2(
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        mode as u16,
        index,
        0 // Неясно, что такое
      )
    };
    return self.error().check(res);
  }
  /// # Парaметры
  /// - `pos`:
  ///   Порядковый номер параметра в запросе (нумерация с 0). Если параметры именованные, то каждое вхождение
  ///   параметра должно привязываться отдельно и может иметь разное значение в каждой привязке.
  /// - `info`:
  ///   Данные для связывания.
  fn bind_by_pos(&self, pos: u32, info: BindInfo, mode: BindMode) -> DbResult<*mut OCIBind> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIBindByPos(
        self.native as *mut OCIStmt,
        &mut handle,
        self.error().native_mut(),
        // В API оракла нумерация с 1, мы же придерживаемся традиционной с 0
        pos + 1,
        // Указатель на данные для получения результата, его размер и тип
        info.ptr as *mut c_void, info.size as i32, info.ty as u16,
        &info.is_null as *const i16 as *mut i16 as *mut c_void,// Массив индикаторов (null/не null, пока не используем)
        ptr::null_mut(),// Массив длин для каждого значения
        ptr::null_mut(),// Массив для column-level return codes

        0, ptr::null_mut(), mode as u32
      )
    };
    try!(self.error().check(res));
    Ok(handle)
  }
  fn bind_by_name(&self, placeholder: &str, info: BindInfo, mode: BindMode) -> DbResult<*mut OCIBind> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIBindByName(
        self.native as *mut OCIStmt,
        &mut handle,
        self.error().native_mut(),
        placeholder.as_ptr(), placeholder.len() as i32,
        // Указатель на данные для получения результата, его размер и тип
        info.ptr as *mut c_void, info.size as i32, info.ty as u16,
        &info.is_null as *const i16 as *mut i16 as *mut c_void,// Массив индикаторов (null/не null, пока не используем)
        ptr::null_mut(),// Массив длин для каждого значения
        ptr::null_mut(),// Массив для column-level return codes

        0, ptr::null_mut(), mode as u32
      )
    };
    try!(self.error().check(res));
    Ok(handle)
  }
  /// # Параметры
  /// - `handle`:
  ///   Описатель связываемого параметра, которому информация буфет предоставляться динамически
  /// - `supplier`:
  ///   Функция, динамически предоставляющая необходимые данные
  fn bind_dynamic<'a, 'b, F>(&'a self, handle: *mut OCIBind, mut supplier: F) -> DbResult<()>
    where F: FnMut(u32, u32, LobPiece) -> (Option<&'a [u8]>, LobPiece, bool) + 'b
  {
    let mut callback: &mut InBindFn = &mut supplier;
    let res = unsafe {
      OCIBindDynamic(
        handle,
        self.error().native_mut(),
        &mut callback as *mut _ as *mut c_void, Some(in_bind_adapter),
        ptr::null_mut(), None
      )
    };
    self.error().check(res)
  }
  /// Ассоциирует с выражением адреса буферов, в которые извлечь данные.
  ///
  /// # Параметры
  /// - `pos`:
  ///   Порядковый номер параметра в запросе (нумерация с 0)
  /// - `dty`:
  ///   Тип данных, которые нужно извлечь
  /// - `buf`:
  ///   Буфер, в который будет записана выходная информация.
  /// - `ind`:
  ///   Переменная, в которую будет записан признак того, что в столбце содержится `NULL`.
  /// - `out_size`:
  ///   Количество байт, записанное в буфер. Не превышает его длину
  fn define(&self, pos: u32, dty: Type, buf: &mut DefineInfo, mode: DefineMode) -> DbResult<()> {
    let res = unsafe {
      OCIDefineByPos(
        self.native as *mut OCIStmt,
        //TODO: Чтобы управлять временем жизни хендла, нужно передать корректный хендл, но тогда его придется
        // самому закрывать. Пока нам это не нужно
        &mut ptr::null_mut(),
        self.error().native_mut(),
        // В API оракла нумерация с 1, мы же придерживаемся традиционной с 0
        pos + 1,
        // Указатель на данные для размещения результата, его размер и тип
        buf.as_ptr(), buf.capacity(), dty as u16,
        &mut buf.is_null as *mut i16 as *mut c_void,// Массив индикаторов (null/не null)
        buf.size_mut(),// Массив длин для каждого значения, которое извлекли из базы
        &mut buf.ret_code,// Массив для column-level return codes
        mode as u32
      )
    };
    self.error().check(res)
  }
  /// Получает количество столбцов, извлеченный в `SELECT`-выражении. Необходимо вызывать после выполнения `SELECT`-запроса,
  /// т.к. до этого момента? или в случае выполнения не `SELECT`-запроса, эта информация недоступна.
  #[inline]
  fn param_count(&self) -> DbResult<u32> {
    self.get_(Attr::ParamCount, self.error())
  }
  /// Получает количество строк, обработанных последним выполненным `INSERT/UPDATE/DELETE` запросом,
  /// или количество строк, полученное последним вызовом fetch для `SELECT` запроса.
  #[inline]
  fn row_count(&self) -> DbResult<u64> {
    self.get_(Attr::RowCount, self.error())
  }
  /// Получает дескриптор с описанием столбца в полученном списке извлеченных `SELECT`-ом столбцов для указанного столбца.
  ///
  /// # Параметры
  /// - `pos`:
  ///   Номер столбца, для которого извлекается информация. Нумерация с 0, в отличие от API оракла, где нумерация идет с 1.
  #[inline]
  fn param_get(&self, pos: u32) -> DbResult<Descriptor<OCIParam>> {
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
  /// Получает информацию о типе выражения.
  pub fn get_type(&self) -> Result<StatementType> {
    let ty: u16 = try!(self.get_(Attr::StmtType, self.error()));

    Ok(unsafe { mem::transmute(ty) })
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
  /// # let env = Environment::default();
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
  /// Выполняет любой запрос. В случае выполнения `INSERT/UPDATE/DELETE` запроса возвращает количество строк,
  /// затронутых запросом (т.е. количество добавленных/обновленных/удаленных строк). Для DDL выражений (например,
  /// `create table`) возвращает `0`.
  ///
  /// Для получения результата от `SELECT` выражения после выполнения данной функции вызовите метод [`get_last_rowset`][1],
  /// либо вместо данного метода воспользуйтесь методом [`query()`][2].
  ///
  /// # OCI вызовы
  /// Для выполнения выражения непосредственно при вызове данной функции используется OCI-вызов [`OCIStmtExecute()`][3].
  /// Для последующего  получения количества затронутых строк используется вызов [`OCIAttrGet()`][4].
  ///
  /// # Запросы к серверу (1)
  /// Непосредственно в момент вызова данной функции выполняется один вызов [`OCIStmtExecute()`][3].
  ///
  /// [1]: #method.get_last_rowset
  /// [2]: #method.query
  /// [3]: https://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17163
  /// [4]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17130
  pub fn execute(&self) -> Result<usize> {
    let count = match try!(self.get_type()) {
      StatementType::SELECT => 0,
      _ => 1,
    };
    try!(self.execute_impl(count, 0, Default::default()));

    Ok(try!(self.row_count()) as usize)
  }
  /// Получает результат последнего исполненного выражения, если это было `SELECT`-выражение и `None` в противном случае.
  pub fn get_last_rowset(&mut self) -> Result<Option<RowSet>> {
    match try!(self.get_type()) {
      StatementType::SELECT => RowSet::new(self).map(|r| Some(r)),
      _ => Ok(None),
    }
  }

  /// Ассоциирует с данным выражением адрес буфера, из которого извлекать данные для заданной переменной.
  ///
  /// # Параметры
  /// - `index`:
  ///   Порядковый номер (нумерация с 0) или символьное имя переменной в запросе.
  /// - `param`:
  ///   Связываемые данные. Должны дожить до вызова [`execute`][1] или [`query`][2].
  ///
  /// # OCI вызовы
  /// При каждом вызове выполняется OCI-вызов [`OCIBindByName()`][3] или [`OCIBindByPos()`][4], в зависимости от
  /// того, какой тип параметра передан в `index`. Когда будет поддержано связывание функций для динамического
  /// предоставления данных, для них будет осуществляться вызов [`OCIBindDynamic()`][5].
  ///
  /// # Запросы к серверу (0)
  /// Ни одна из вызываемых функций не выполняет запросов к серверу.
  ///
  /// # Unsafe
  /// Функция небезопасная по той причине, что параметр `param` должен дожить до вызова [`execute`][1] или [`query`][2].
  /// К сожалению, пока неясно, как заставить компилятор форсировать данное требование.
  ///
  /// [1]: #method.execute
  /// [2]: #method.query
  /// [3]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI17140
  /// [4]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI17141
  /// [5]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI17142
  pub unsafe fn bind<'i, 'p, I, P>(&mut self, index: I, param: P) -> Result<()>
    where I: Into<BindIndex<'i>>,
          P: Into<BindInfo<'p>> + 'p
  {
    let info = param.into();
    try!(match index.into() {
      BindIndex::Name(name) => self.bind_by_name(name, info, BindMode::default()),
      BindIndex::Index(pos) => self.bind_by_pos(pos as u32, info, BindMode::default()),
    });
    Ok(())
  }
}
impl<'conn, 'key> Drop for Statement<'conn, 'key> {
  fn drop(&mut self) {
    let keyPtr = self.key.map_or(0 as *const u8, |x| x.as_ptr());
    let keyLen = self.key.map_or(0 as u32      , |x| x.len() as u32);
    let res = unsafe { OCIStmtRelease(self.native as *mut OCIStmt, self.error().native_mut(), keyPtr, keyLen, 0) };

    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.error().check(res);//.expect("OCIStmtRelease");
  }
}
impl<'conn, 'key> AttrHolder<OCIStmt> for Statement<'conn, 'key> {
  fn holder_type() -> u32 {
    ::ffi::types::Handle::Stmt as u32
  }
  fn native(&self) -> *const OCIStmt {
    self.native
  }
}

impl<'conn, 'key> super::StatementPrivate for Statement<'conn, 'key> {
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: Syntax) -> DbResult<Statement<'c, 'k>> {
    let mut stmt = ptr::null_mut();
    let keyPtr = key.map_or(0 as *const u8, |x| x.as_ptr());
    let keyLen = key.map_or(0 as u32      , |x| x.len() as u32);
    let res = unsafe {
      OCIStmtPrepare2(
        conn.context.native_mut(),
        &mut stmt as *mut *mut OCIStmt,
        conn.error().native_mut(),
        // Текст SQL запроса
        sql.as_ptr(), sql.len() as u32,
        // Ключ кеширования, по которому достанется запрос, если он был закеширован
        keyPtr, keyLen,
        syntax as u32, CachingMode::Default as u32
      )
    };
    return match res {
      0 => Ok(Statement { conn: conn, native: stmt, key: key }),
      e => Err(conn.error().decode(e)),
    };
  }
}
trait RowSetPrivate<'stmt> : Sized {
  /// Создает набор из выражения. Запоминает описание столбцов выражения
  fn new(stmt: &'stmt Statement) -> Result<Self>;
}