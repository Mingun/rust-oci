
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_int, c_void, c_char, c_uchar, c_uint, c_ushort};
use std::ptr;
use std::slice;
use super::Error;
use super::Result;
use super::ConnectParams;

mod base;
mod types;
mod native;

pub use self::types::{CreateMode, AttachMode, MallocFn, ReallocFn, FreeFn};
use self::native::*;
use self::base::{Handle, Descriptor, Env};
use self::base::AttrHolder;

//-------------------------------------------------------------------------------------------------
unsafe fn attr_get<T: AttrHandle>(handle: *const T, value: *mut c_void, size: &mut c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
  let res = OCIAttrGet(
    handle as *const c_void, T::ID as c_uint,
    value, size, attrtype as c_uint,
    err.native_mut()
  );
  return err.check(res);
}
fn attr_get_c_uint<T: AttrHandle>(handle: *const T, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<c_uint> {
  let mut res: c_uint = 0;
  let ptr = &mut res as *mut c_uint;
  try!(unsafe { attr_get(handle, ptr as *mut c_void, &mut 0, attrtype, err) });

  Ok(res)
}
unsafe fn attr_set<T: AttrHandle>(handle: *mut T, value: *mut c_void, size: c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
  let res = OCIAttrSet(
    handle as *mut c_void, T::ID as c_uint,
    value, size, attrtype as c_uint,
    err.native_mut()
  );
  return err.check(res);
}
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
#[derive(Debug)]
pub struct Environment<'e> {
  env: Env<'e>,
  error: Handle<OCIError>,
}
impl<'e> Environment<'e> {
  pub fn new(mode: types::CreateMode) -> Result<Self> {
    let env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.handle(env.native_mut()));

    Ok(Environment { env: env, error: err })
  }
  pub fn connect<P: Into<ConnectParams>>(&self, params: P) -> Result<Connection> {
    let p = params.into();
    Connection::new(&self, &p.dblink, p.mode, &p.username, &p.password)
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    self.env.handle(self.error.native_mut())
  }
  fn descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    Descriptor::new(&self)
  }
  fn error(&self) -> &Handle<OCIError> {
    &self.error
  }
}
//-------------------------------------------------------------------------------------------------
/// Хранит автоматически закрываемый хендл `OCIServer`, предоставляющий доступ к базе данных
#[derive(Debug)]
struct Server<'env> {
  env: &'env Environment<'env>,
  handle: Handle<OCIServer>,
  mode: types::AttachMode,
}
impl<'env> Server<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode) -> Result<Server<'e>> {
    let server: Handle<OCIServer> = try!(env.handle());
    let res = unsafe {
      OCIServerAttach(
        server.native_mut(), env.error.native_mut(),
        dblink.as_ptr(), dblink.len() as c_int,
        mode as c_uint
      )
    };
    return match res {
      0 => Ok(Server { env: env, handle: server, mode: mode }),
      e => Err(env.error.decode(e))
    };
  }
  fn error(&self) -> &Handle<OCIError> {
    self.env.error()
  }
}
impl<'env> Drop for Server<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCIServerDetach(
        self.handle.native_mut(),
        self.error().native_mut(),
        self.mode as c_uint
      )
    };
    self.error().check(res).expect("OCIServerDetach");
  }
}
//-------------------------------------------------------------------------------------------------
/// Представляет соединение к базе данных, с определенным пользователем и паролем.
/// Соединение зависит от окружения, создавшего его, таким образом, окружение является менеджером
/// соединений. При уничтожении окружения все соединения закрываются, а незакоммиченные транзакции
/// в них откатываются.
#[derive(Debug)]
pub struct Connection<'env> {
  server: Server<'env>,
  context: Handle<OCISvcCtx>,
  session: Handle<OCISession>,
}
impl<'env> Connection<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode, username: &str, password: &str) -> Result<Connection<'e>> {
    let server = try!(Server::new(env, dblink, mode));
    let mut context: Handle<OCISvcCtx > = try!(env.handle());
    let mut session: Handle<OCISession> = try!(env.handle());

    // Ассоциируем имя пользователя и пароль с сессией
    try!(session.set_str(username, types::Attr::Username, &env.error));
    try!(session.set_str(password, types::Attr::Password, &env.error));

    // Ассоциируем сервер с контекстом и осуществляем подключение
    try!(context.set_handle(&server.handle, types::Attr::Server, &env.error));
    let res = unsafe {
      OCISessionBegin(
        context.native_mut(),
        env.error.native_mut(),
        session.native_mut(),
        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        types::CredentialMode::Rdbms as c_uint,
        types::AuthMode::Default as c_uint
      )
    };
    try!(env.error.check(res));
    try!(context.set_handle(&session, types::Attr::Session, &env.error));

    Ok(Connection { server: server, context: context, session: session })
  }
  fn error(&self) -> &Handle<OCIError> {
    self.server.error()
  }

  pub fn prepare<'c>(&'c self, sql: &str) -> Result<Statement<'c, 'c>> {
    Statement::new(&self, sql, None, types::Syntax::Native)
  }
}
impl<'env> Drop for Connection<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCISessionEnd(
        self.context.native_mut(),
        self.error().native_mut(),
        self.session.native_mut(),
        types::AuthMode::Default as c_uint
      )
    };
    self.error().check(res).expect("OCISessionEnd");
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
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: types::Syntax) -> Result<Statement<'c, 'k>> {
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
        syntax as c_uint, types::CachingMode::Default as c_uint
      )
    };
    return match res {
      0 => Ok(Statement { conn: conn, native: stmt, key: key }),
      e => Err(conn.error().decode(e)),
    };
  }
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
  fn execute(&self, count: c_uint, offset: c_uint, mode: types::ExecuteMode) -> Result<()> {
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
  fn fetch(&self, count: c_uint, mode: types::FetchMode, index: c_int) -> Result<()> {
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
  fn param_count(&self) -> Result<c_uint> {
    attr_get_c_uint(self.native, types::Attr::ParamCount, self.error())
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

//-------------------------------------------------------------------------------------------------
/// Структура для представления колонки базы данных из списка выбора
#[derive(Debug)]
pub struct Column {
  /// Номер столбца
  pub pos: usize,
  pub type_: types::Type,
  /// Название колонки в базе данных
  pub name: String,
  /// Ширина колонки в байтах
  pub size: usize,
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_: c_ushort = try!(desc.get_(types::Attr::DataType, err));
    let name  = try!(desc.get_str(types::Attr::Name, err));
    //let ischar= try!(desc.get_(types::Attr::CharUsed, err));
    //let size : c_uint  = try!(desc.get_(types::Attr::CharSize, err));
    let size : c_uint = try!(desc.get_(types::Attr::DataSize, err));

    Ok(Column { pos: pos, name: name, size: size as usize, type_: unsafe { mem::transmute(type_ as u16) } })
  }
}