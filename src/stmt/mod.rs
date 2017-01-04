//! Содержит определение подготовленных выражений, которые используются для эффективного исполнения запросов,
//! и структур, связанных с ними.
mod storage;

use std::mem;
use std::os::raw::{c_int, c_short, c_void, c_uchar, c_uint, c_ushort};
use std::ptr;

use {Connection, Result};
use error::Error;
use error::DbError::NoData;
use types::{FromDB, Type, Syntax};

use ffi::{Descriptor, Handle};
use ffi::attr::AttrHolder;
use ffi::native::{OCIParam, OCIStmt, OCIBind, OCIError};// FFI типы
use ffi::native::{OCIParamGet, OCIStmtExecute, OCIStmtRelease, OCIStmtPrepare2, OCIStmtFetch2, OCIBindByPos, OCIBindByName, OCIDefineByPos};// FFI функции
use ffi::native::ParamHandle;// Типажи для безопасного моста к FFI
use ffi::types::Attr;
use ffi::types::{DefineMode, CachingMode, ExecuteMode, FetchMode};

use self::storage::DefineInfo;

//-------------------------------------------------------------------------------------------------
fn param_get<'d, T: ParamHandle>(handle: *const T, pos: c_uint, err: &Handle<OCIError>) -> Result<Descriptor<'d, OCIParam>> {
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
  /// Название колонки в списке выбора (т.е. либо название колонки в базе дынных, либо ее псевдоним).
  pub name: String,
  /// Ширина колонки в байтах. Показывает, сколько байт максимум может занимать значение колонки,
  /// а не занимаемый реально данными объем.
  pub size: usize,
  pub precision: usize,
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_: c_ushort = try!(desc.get_(Attr::DataType, err));
    let name  = try!(desc.get_str(Attr::Name, err));
    //let ischar= try!(desc.get_(Attr::CharUsed, err));
    //let size : c_uint  = try!(desc.get_(Attr::CharSize, err));
    let size : c_uint = try!(desc.get_(Attr::DataSize, err));
    let prec : c_uint = try!(desc.get_(Attr::Precision, err));

    Ok(Column { pos: pos, name: name, size: size as usize, type_: unsafe { mem::transmute(type_ as u16) }, precision: prec as usize })
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
  fn execute_impl(&self, count: c_uint, offset: c_uint, mode: ExecuteMode) -> Result<()> {
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
  fn fetch(&self, count: c_uint, mode: FetchMode, index: c_int) -> Result<()> {
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
  fn bind_by_pos(&self, pos: c_uint, value: *mut c_void, size: c_int, dty: Type) -> Result<Handle<OCIBind>> {
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
  fn bind_by_name(&self, placeholder: &str, value: *mut c_void, size: c_int, dty: Type) -> Result<Handle<OCIBind>> {
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
  fn define(&self, pos: usize, dty: Type, buf: &mut DefineInfo, mode: DefineMode) -> Result<()> {
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
  fn param_count(&self) -> Result<c_uint> {
    self.get_(Attr::ParamCount, self.error())
  }
  /// Получает дескриптор с описанием столбца в полученном списке извлеченных `SELECT`-ом столбцов для указанного столбца.
  ///
  /// # Параметры
  /// - pos:
  ///   Номер столбца, для которого извлекается информация. Нумерация с 0, в отличие от API оракла, где нумерация идет с 1.
  #[inline]
  fn param_get(&self, pos: c_uint) -> Result<Descriptor<OCIParam>> {
    param_get(self.native, pos + 1, self.error())
  }

  /// Возвращает соединение, из которого было подготовлено данные выражение.
  #[inline]
  pub fn connection(&self) -> &Connection {
    self.conn
  }
  /// Получает информацию о списке выбора `SELECT`-выражения.
  pub fn columns(&self) -> Result<Vec<Column>> {
    let cnt = try!(self.param_count());
    let mut vec = Vec::with_capacity(cnt as usize);
    for i in 0..cnt {
      vec.push(try!(Column::new(i as usize, try!(self.param_get(i)), self.error())));
    }
    Ok(vec)
  }
  pub fn query(&self) -> Result<RowSet> {
    try!(self.execute_impl(0, 0, Default::default()));

    RowSet::new(self)
  }
  pub fn execute(&self) -> Result<()> {
    self.execute_impl(1, 0, Default::default())
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
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: Syntax) -> Result<Statement<'c, 'k>> {
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
}
impl<'d> Row<'d> {
  fn new(rs: &'d RowSet) -> Result<Self> {
    let mut data: Vec<DefineInfo> = Vec::with_capacity(rs.columns.len());

    for c in &rs.columns {
      data.push(try!(DefineInfo::new(rs.stmt, c)));
      // unwrap делать безопасно, т.к. мы только что вставили в массив данные
      try!(rs.stmt.define(c.pos, c.bind_type(), data.last_mut().unwrap(), Default::default()));
    }

    Ok(Row { rs: rs, data: data })
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
  pub fn get<T: FromDB, I: RowIndex>(&self, index: I) -> Result<Option<T>> {
    let col = try!(self.column(index));
    self.data[col.pos].to(col.bind_type(), self.rs.stmt.connection())
  }
}
/// Набор результатов, полученный при выполнении `SELECT` выражения. Итерация по набору позволяет получить данные,
/// извлеченные из базы данных.
#[derive(Debug)]
pub struct RowSet<'stmt> {
  /// Выражение, выполнение которого дало данный набор результатов
  stmt: &'stmt Statement<'stmt, 'stmt>,
  /// Список колонок, которые извлекали из базы данных
  columns: Vec<Column>,
}
/// Набор строк, полученный в результате выполнения `SELECT`-выражения. В настоящий момент при итерации по набору
/// получается одна строка за раз, с выполнением обращения к серверу.
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
}
impl<'stmt> IntoIterator for &'stmt RowSet<'stmt> {
  type Item = Row<'stmt>;
  type IntoIter = RowIter<'stmt>;

  fn into_iter(self) -> Self::IntoIter {
    RowIter { rs: self }
  }
}

/// Однонаправленный итератор по результатам `SELECT`-а.
#[derive(Debug)]
pub struct RowIter<'rs> {
  /// Набор строк, по которому осуществляется итерация
  rs: &'rs RowSet<'rs>,
}
impl<'rs> Iterator for RowIter<'rs> {
  type Item = Row<'rs>;

  fn next(&mut self) -> Option<Self::Item> {
    // Подготавливаем место в памяти для извлечения данных.
    // TODO: его можно переиспользовать, незачем создавать каждый раз.
    let r = Row::new(self.rs).expect("Row::new failed");
    match self.rs.stmt.fetch(1, Default::default(), 0) {
      Ok(_) => Some(r),
      Err(Error::Db(NoData)) => None,
      Err(e) => panic!("`fetch` failed: {:?}", e)
    }
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
