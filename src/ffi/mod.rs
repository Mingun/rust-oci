
use std::os::raw::{c_int, c_uint};
use std::ptr;
use {ConnectParams, Credentials, Result};

mod attr;
mod base;
mod stmt;
mod types;
mod native;

pub use self::types::{CreateMode, AttachMode, AuthMode, MallocFn, ReallocFn, FreeFn};
pub use self::stmt::{Column, Statement};
use self::native::*;
use self::base::{Handle, Descriptor, Env};
use self::base::AttrHolder;
use self::stmt::StatementPrivate;

//-------------------------------------------------------------------------------------------------
/// Окружение представляет собой менеджер соединений к базе. При разрушении окружения
/// все открытые соединения автоматически закрываются а незавершенные транзакции в них
/// откатываются.
#[derive(Debug)]
pub struct Environment<'e> {
  env: Env<'e>,
  /// Хендл для приема ошибок от нативных вызовов оракла. Позволяет затем получить код ошибки
  /// и ее описание.
  error: Handle<OCIError>,
}
impl<'e> Environment<'e> {
  pub fn new(mode: types::CreateMode) -> Result<Self> {
    let env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.handle(env.native_mut()));

    Ok(Environment { env: env, error: err })
  }
  /// Осуществляет подключение к базе данных с указанными параметрами
  pub fn connect<P: Into<ConnectParams>>(&self, params: P) -> Result<Connection> {
    Connection::new(&self, &params.into())
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
  fn new<'e>(env: &'e Environment, dblink: Option<&str>, mode: types::AttachMode) -> Result<Server<'e>> {
    let server: Handle<OCIServer> = try!(env.handle());
    let (ptr, len) = match dblink {
      Some(db) => (db.as_ptr(), db.len()),
      None => (ptr::null(), 0)
    };
    let res = unsafe {
      OCIServerAttach(
        server.native_mut(), env.error.native_mut(),
        ptr, len as c_int,
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
/// соединений. При уничтожении окружения все соединения закрываются, а не закоммиченные транзакции
/// в них откатываются.
#[derive(Debug)]
pub struct Connection<'env> {
  /// Хендл сервера, к которому будут направляться запросы. Несколько пользователей (подключений)
  /// могут одновременно работать с одним сервером через общий хендл. В настоящий момент это не
  /// поддерживается, каждое подключение использует свое сетевое соединение к серверу.
  server: Server<'env>,
  context: Handle<OCISvcCtx>,
  session: Handle<OCISession>,
  /// Режим аутетификации, который использовался при создании соединения. Необходим при закрытии
  auth_mode: types::AuthMode,
}
impl<'env> Connection<'env> {
  fn new<'e>(env: &'e Environment, params: &ConnectParams) -> Result<Connection<'e>> {
    let server = try!(Server::new(env, Some(&params.dblink), params.attach_mode));
    let mut context: Handle<OCISvcCtx > = try!(env.handle());
    let mut session: Handle<OCISession> = try!(env.handle());

    let credMode = match params.credentials {
      Credentials::Rdbms { ref username, ref password } => {
        // Ассоциируем имя пользователя и пароль с сессией.
        // Надо отметить, что эти атрибуты сохраняются после закрытия сессии и при переподключении
        // можно их заново не устанавливать.
        try!(session.set_str(username, types::Attr::Username, &env.error));
        try!(session.set_str(password, types::Attr::Password, &env.error));

        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        types::CredentialMode::Rdbms
      },
      Credentials::Ext => types::CredentialMode::Ext,
    };

    // Ассоциируем сервер с контекстом и осуществляем подключение
    try!(context.set_handle(&server.handle, types::Attr::Server, &env.error));
    let res = unsafe {
      OCISessionBegin(
        context.native_mut(),
        env.error.native_mut(),
        session.native_mut(),
        credMode as c_uint,
        params.auth_mode as c_uint
      )
    };
    try!(env.error.check(res));
    try!(context.set_handle(&session, types::Attr::Session, &env.error));

    Ok(Connection { server: server, context: context, session: session, auth_mode: params.auth_mode })
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
        self.auth_mode as c_uint
      )
    };
    self.error().check(res).expect("OCISessionEnd");
  }
}