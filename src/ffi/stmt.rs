
use std::convert::{From, Into};
use std::mem;
use std::os::raw::{c_int, c_short, c_void, c_uchar, c_uint, c_ushort};
use std::ptr;
use std::slice;
use Result;
use types::{FromDB, Type};
use super::base::{Descriptor, Handle};
use super::base::AttrHolder;
use super::native::*;
use super::types::Attr;
use super::types::{DefineMode, CachingMode, ExecuteMode, FetchMode, Syntax};
use super::Connection;

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
  /// Номер столбца
  pub pos: usize,
  pub type_: Type,
  /// Название колонки в базе данных
  pub name: String,
  /// Ширина колонки в байтах
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
  fn error(&self) -> &Handle<OCIError> {
    self.conn.error()
  }
  /// # Параметры
  /// - count:
  ///   * Для `select` выражений это количество строк, которые нужно извлечть prefetch-ем, уже в момент выполнения
  ///     запроса (т.е. сервер БД вернет их, не дожидаясь вызова `OCIStmtFetch2`). Если prefetch не нужен, то должно
  ///     быть равно `0`.
  ///   * Для не-`select` выражений это номер последнего элемента в буфере данных со связанными параметрами, которые
  ///     нужно использовать при выполнении данной операции
  /// - offset:
  ///   Смещение с буфере со связанными переменными, с которого необходимо начать выполнение 
  fn execute(&self, count: c_uint, offset: c_uint, mode: ExecuteMode) -> Result<()> {
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
  ///   Порядковый момер параметра в запросе (нумерация с 0)
  /// - dty:
  ///   Тип данных, которые нужно извлечь
  /// - buf:
  ///   Буфер, в который будет записана выходная информация.
  /// - ind:
  ///   Переменная, в которую будет записан признак того, что в столбце содержится `NULL`.
  /// - out_size:
  ///   Количество байт, записанное в буфер. Не превышает его длину
  fn define(&self, pos: usize, dty: Type, buf: &mut Storage, mode: DefineMode) -> Result<()> {
    let res = unsafe {
      OCIDefineByPos(
        self.native as *mut OCIStmt,
        //TODO: Чтобы управлять временем жизни хендла, нужно передать корректный хендл, но тогда его придется
        // самому закрывать. Пока нам это не нужно
        &mut ptr::null_mut(),
        self.error().native_mut(),
        // В API оракла нумерация с 1, мы же придерживемся традиционной с 0
        (pos + 1) as c_uint,
        // Указатель на данные для размещения результата, его размер и тип
        buf.ptr as *mut c_void, buf.capacity as c_int, dty as c_ushort,
        (&mut buf.is_null as *mut c_short) as *mut c_void,// Массив индикаторов (null/не null)
        &mut buf.size as *mut c_ushort,// Массив длин для каждого значения, которое извлекли из базы
        &mut buf.ret_code as *mut c_ushort,// Массив для column-level return codes
        mode as c_uint
      )
    };
    self.error().check(res)
  }
  fn param_count(&self) -> Result<c_uint> {
    self.get_(Attr::ParamCount, self.error())
  }
  fn param_get(&self, pos: c_uint) -> Result<Descriptor<OCIParam>> {
    param_get(self.native, pos, self.error())
  }

  pub fn columns(&self) -> Result<Vec<Column>> {
    let cnt = try!(self.param_count());
    let mut vec = Vec::with_capacity(cnt as usize);
    for i in 0..cnt {
      vec.push(try!(Column::new(i as usize, try!(self.param_get(i+1)), self.error())));
    }
    Ok(vec)
  }
  pub fn query(&self) -> Result<RowSet> {
    try!(self.execute(0, 0, Default::default()));

    Ok(RowSet { stmt: &self })
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
    super::types::Handle::Stmt as c_uint
  }
  fn native(&self) -> *const OCIStmt {
    self.native
  }
}

pub trait StatementPrivate {
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
impl<'conn, 'key> StatementPrivate for Statement<'conn, 'key> {}

/// Хранилище буферов для биндинга результатов, извлекаемых из базы, для одной колонки
#[derive(Debug)]
struct Storage {
    ptr: *mut u8,
    capacity: usize,
    size: c_ushort,
    /// Возможные значения:
    /// * `-2`  The length of the item is greater than the length of the output variable; the item has been truncated. Additionally,
    ///         the original length is longer than the maximum data length that can be returned in the sb2 indicator variable.
    /// * `-1`  The selected value is null, and the value of the output variable is unchanged.
    /// * `0`   Oracle Database assigned an intact value to the host variable.
    /// * `>0`  The length of the item is greater than the length of the output variable; the item has been truncated. The positive
    ///         value returned in the indicator variable is the actual length before truncation.
    is_null: c_short,
    ret_code: c_ushort,
}
impl Storage {
  #[inline]
  fn to_vec(&self) -> Vec<u8> {
    unsafe { Vec::from_raw_parts(self.ptr, self.size as usize, self.capacity) }
  }
  #[inline]
  fn as_slice(&self) -> Option<&[u8]> {
    match self.is_null {
      0 => Some(unsafe { slice::from_raw_parts(self.ptr, self.size as usize) }),
      _ => None
    }
  }
  #[inline]
  fn to<T: FromDB + ?Sized>(&self, ty: Type) -> Result<Option<&T>> {
    match self.as_slice() {
      Some(ref slice) => T::from_db(ty, slice).map(|r| Some(r)),
      None => Ok(None),
    }
  }
}
impl From<Vec<u8>> for Storage {
  fn from(mut backend: Vec<u8>) -> Self {
    let res = Storage { ptr: backend.as_mut_ptr(), size: 0, capacity: backend.capacity(), is_null: 0, ret_code: 0 };
    // Вектор уходит в небытие, чтобы он не забрал память с собой, забываем его
    mem::forget(backend);
    res
  }
}
impl Into<Option<Vec<u8>>> for Storage {
  fn into(self) -> Option<Vec<u8>> {
    match self.is_null {
      0 => Some(self.to_vec()),
      _ => None
    }
  }
}
impl Drop for Storage {
  fn drop(&mut self) {
    // Освобождаем память деструктором вектора, ведь память была выделена его конструктором
    drop(self.to_vec());
    self.ptr = ptr::null_mut();
    self.capacity = 0;
    self.size = 0;
    self.is_null = 0;
    self.ret_code = 0;
  }
}
#[derive(Debug)]
pub struct Row {
  /// Массив данных для каждой колонки.
  data: Vec<Storage>,
}
impl Row {
  fn new(stmt: &Statement) -> Result<Self> {
    let columns = try!(stmt.columns());
    let mut data: Vec<Storage> = Vec::with_capacity(columns.len());

    for c in columns {
      data.push(Vec::with_capacity(c.size).into());
      try!(stmt.define(c.pos, c.bind_type(), &mut data.last_mut().unwrap(), Default::default()));
    }

    Ok(Row { data: data })
  }
  pub fn get<'a, T: FromDB + ?Sized>(&'a self, col: &Column) -> Result<Option<&'a T>> {
    self.data[col.pos].to(col.bind_type())
  }
}
#[derive(Debug)]
pub struct RowSet<'s> {
  stmt: &'s Statement<'s, 's>,
}

impl<'s> Iterator for RowSet<'s> {
  type Item = Row;

  fn next(&mut self) -> Option<Self::Item> {
    let r = Row::new(self.stmt).expect("Row::new failed");
    self.stmt.fetch(1, Default::default(), 0).expect("`fetch` failed");
    Some(r)
  }
}
