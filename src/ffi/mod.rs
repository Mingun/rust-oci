
use std::os::raw::{c_int, c_void, c_uchar, c_uint};
use std::ptr;
use super::Error;
use super::Result;
use super::ConnectParams;

mod types;
mod native;

pub use self::types::{CreateMode, AttachMode, MallocFn, ReallocFn, FreeFn};
use self::native::*;

//-------------------------------------------------------------------------------------------------
fn check(native: c_int) -> Result<()> {
  return match native {
    0 => Ok(()),
    e => Err(Error(e))
  };
}
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый хендл на ресурсы оракла
#[derive(Debug)]
struct Handle<T: HandleType> {
  native: *mut T,
}
impl<T: HandleType> Handle<T> {
  fn new(env: &Env) -> Result<Handle<T>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIHandleAlloc(
        env.native as *const c_void,
        &mut handle, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделеное под них место
      )
    };
    return match res {
      0 => Ok(Handle { native: handle as *mut T }),
      e => Err(Error(e))
    };
  }
  fn set(&mut self, value: *mut c_void, size: c_uint, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    let res = unsafe {
      OCIAttrSet(
        self.native as *mut c_void, T::ID as c_uint,
        value, size, attrtype as c_uint,
        errhp.native
      )
    };
    return check(res);
  }
  /// Устанавливает строковый атрибут хендлу
  fn set_str(&mut self, value: &str, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    self.set(value.as_ptr() as *mut c_void, value.len() as c_uint, attrtype, errhp)
  }
  /// Устанавливает хендл-атрибут хендлу
  fn set_handle<U: HandleType>(&mut self, value: &Handle<U>, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    self.set(value.native as *mut c_void, 0, attrtype, errhp)
  }
}
impl<T: HandleType> Drop for Handle<T> {
  fn drop(&mut self) {
    let res = unsafe { OCIHandleFree(self.native as *mut c_void, T::ID as c_uint) };
    check(res).expect("OCIHandleFree");
  }
}
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый дескриптор ресурсов оракла
#[derive(Debug)]
struct Descriptor<T: DescriptorType> {
  native: *mut T,
}
impl<T: DescriptorType> Descriptor<T> {
  fn new(env: &Env) -> Result<Descriptor<T>> {
    let mut desc = ptr::null_mut();
    let res = unsafe {
      OCIDescriptorAlloc(
        env.native as *const c_void,
        &mut desc, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделеное под них место
      )
    };
    return match res {
      0 => Ok(Descriptor { native: desc as *mut T }),
      e => Err(Error(e))
    };
  }
}
impl<T: DescriptorType> Drop for Descriptor<T> {
  fn drop(&mut self) {
    let res = unsafe { OCIDescriptorFree(self.native as *mut c_void, T::ID as c_uint) };
    check(res).expect("OCIDescriptorFree");
  }
}
//-------------------------------------------------------------------------------------------------
/// Автоматически закрываемый хендл окружения оракла
#[derive(Debug)]
struct Env {
  native: *mut OCIEnv,
  mode: types::CreateMode,
}
impl Env {
  fn new(mode: types::CreateMode) -> Result<Self> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIEnvNlsCreate(
        &mut handle, // сюда записывается результат
        mode as c_uint,
        0 as *mut c_void, // Контекст для функций управления памятью.
        None, None, None, // Функции управления памятью
        0, 0 as *mut *mut c_void,// размер пользовательских данных и указатель на выделеное под них место
        0, 0// Параметры локализации для типов CHAR и NCHAR. 0 - использовать настройку NLS_LANG
      )
    };
    return match res {
      0 => Ok(Env { native: handle, mode: mode }),
      e => Err(Error(e))
    };
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    Handle::new(&self)
  }
  fn descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    Descriptor::new(&self)
  }
}
impl Drop for Env {
  fn drop(&mut self) {
    let res = unsafe { OCITerminate(self.mode as c_uint) };
    check(res).expect("OCITerminate");
  }
}
//-------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Environment {
  env: Env,
  error: Handle<OCIError>,
}
impl Environment {
  pub fn new(mode: types::CreateMode) -> Result<Self> {
    let env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.handle());

    Ok(Environment { env: env, error: err })
  }
  pub fn connect<P: Into<ConnectParams>>(&self, params: P) -> Result<Connection> {
    let p = params.into();
    Connection::new(&self, &p.dblink, p.mode, &p.username, &p.password)
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    self.env.handle()
  }
  fn descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    self.env.descriptor()
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.error.native
  }
}
impl Drop for Environment {
  fn drop(&mut self) {}
}
//-------------------------------------------------------------------------------------------------
/// Хранит автоматически закрываемый хендл `OCIServer`, предоставляющий доступ к базе данных
#[derive(Debug)]
struct Server<'env> {
  env: &'env Environment,
  handle: Handle<OCIServer>,
  mode: types::AttachMode,
}
impl<'env> Server<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode) -> Result<Server<'e>> {
    let server: Handle<OCIServer> = try!(env.handle());
    let res = unsafe {
      OCIServerAttach(
        server.native, env.error.native,
        dblink.as_ptr(), dblink.len() as c_int,
        mode as c_uint
      )
    };
    return match res {
      0 => Ok(Server { env: env, handle: server, mode: mode }),
      e => Err(Error(e))
    };
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.env.errorHandle()
  }
}
impl<'env> Drop for Server<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCIServerDetach(
        self.handle.native, self.errorHandle(),
        self.mode as c_uint
      )
    };
    check(res).expect("OCIServerDetach");
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
        context.native,
        env.error.native,
        session.native,
        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        types::CredentialMode::Rdbms as c_uint,
        types::AuthMode::Default as c_uint
      )
    };
    try!(check(res));
    try!(context.set_handle(&session, types::Attr::Session, &env.error));

    Ok(Connection { server: server, context: context, session: session })
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.server.errorHandle()
  }

  pub fn prepare<'c>(&'c self, sql: &str) -> Result<Statement<'c, 'c>> {
    Statement::new(&self, sql, None, types::Syntax::Native)
  }
}
impl<'env> Drop for Connection<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCISessionEnd(
        self.context.native,
        self.errorHandle(),
        self.session.native,
        types::AuthMode::Default as c_uint
      )
    };
    check(res).expect("OCISessionEnd");
  }
}
//-------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Statement<'conn, 'key> {
  /// Соединение, которое подготовило данное выражение
  conn: &'conn Connection<'conn>,
  /// Внутренний указатель оракла на подготовленное выражение
  native: *mut OCIStmt,
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
        conn.context.native,
        &mut stmt as *mut *mut OCIStmt,
        conn.errorHandle(),
        // Текст SQL запроса
        sql.as_ptr() as *const c_uchar, sql.len() as c_uint,
        // Ключ кеширования, по которому достанется запрос, если он был закеширован
        keyPtr, keyLen,
        syntax as c_uint, types::CachingMode::Default as c_uint
      )
    };
    return match res {
      0 => Ok(Statement { conn: conn, native: stmt, key: key }),
      e => Err(Error(e)),
    };
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.conn.errorHandle()
  }
}
impl<'conn, 'key> Drop for Statement<'conn, 'key> {
  fn drop(&mut self) {
    let keyPtr = self.key.map_or(0 as *const c_uchar, |x| x.as_ptr() as *const c_uchar);
    let keyLen = self.key.map_or(0 as c_uint        , |x| x.len()  as c_uint);
    let res = unsafe { OCIStmtRelease(self.native, self.errorHandle(), keyPtr, keyLen, 0) };
    check(res).expect("OCIStmtRelease");
  }
}