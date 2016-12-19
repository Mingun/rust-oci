
use std::mem;
use std::os::raw::{c_longlong, c_int, c_void, c_uchar, c_uint, c_ushort};
use std::ptr;

use super::super::Result;
use super::base::{Descriptor, Handle};
use super::base::AttrHolder;
use super::native::*;
use super::types::Attr;
use super::types::{Type, CachingMode, ExecuteMode, FetchMode, Syntax};
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
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_: c_ushort = try!(desc.get_(Attr::DataType, err));
    let name  = try!(desc.get_str(Attr::Name, err));
    //let ischar= try!(desc.get_(Attr::CharUsed, err));
    //let size : c_uint  = try!(desc.get_(Attr::CharSize, err));
    let size : c_uint = try!(desc.get_(Attr::DataSize, err));

    Ok(Column { pos: pos, name: name, size: size as usize, type_: unsafe { mem::transmute(type_ as u16) } })
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
  fn bind_by_name(&self, placeholder: &str, value: *mut c_void, size: c_longlong, dty: Type) -> Result<()> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIBindByName2(
        self.native as *mut OCIStmt,
        &mut handle,
        self.error().native_mut(),
        placeholder.as_ptr() as *const c_uchar, placeholder.len() as c_int,
        // Указатель на данные для размещения результата, его размер и тип
        value, size, dty as c_ushort,
        ptr::null_mut(),// Массив индикаторов (null/не null, пока не используем)
        ptr::null_mut(),// Массив длин для каждого значения
        ptr::null_mut(),// Массив для column-level return codes

        0, ptr::null_mut(), 0
      )
    };
    return self.error().check(res);
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
  pub fn query(&self) -> Result<()> {
    self.execute(0, 0, Default::default())
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