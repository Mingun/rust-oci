#![feature(associated_consts)]
#![allow(non_snake_case)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;
extern crate num_traits;

pub mod error;
pub mod params;
pub mod types;
mod ffi;

/// Тип результата, возвращаемый всеми функциями библиотеки, которые могут привести к ошибке.
/// В большинстве случаев библиотека никогда не генерирует панику, всегда возращая ошибочный
/// результат в виде ошибке. Немногочисленные исключения документированы особо, и существуют
/// потому, что внешнее по отношению к библиотеке API не позволяет вернуть ошибку (Например,
/// из реализации типажа [`Drop`][1]).
///
/// [1]: https://doc.rust-lang.org/std/ops/trait.Drop.html
pub type Result<T> = std::result::Result<T, error::Error>;
// Реэкспорты
pub use ffi::stmt::{Column, Statement, RowSet, Row};

use std::os::raw::c_uint;

use params::{ConnectParams, Credentials};
use types::{CreateMode, AuthMode, Syntax};

use ffi::{Env, Server, Handle, Descriptor};
use ffi::types::{Attr, CredentialMode};
use ffi::native::{OCISvcCtx, OCISession, OCIError};// FFI типы
use ffi::native::{OCISessionBegin, OCISessionEnd};// FFI функции
use ffi::native::{HandleType, DescriptorType};// Типажи для безопасного моста к FFI

// Для того, чтобы пользоваться функциями типажей, они должны быть в области видимости
use ffi::attr::AttrHolder;
use ffi::stmt::StatementPrivate;

//-------------------------------------------------------------------------------------------------
/// Окружение представляет собой менеджер соединений к базе. При разрушении окружения
/// все открытые соединения автоматически закрываются а незавершенные транзакции в них
/// откатываются.
#[derive(Debug)]
pub struct Environment<'e> {
  /// Автоматически закрываемый враппер над низкоуровневыми функциями работы с окружением Oracle
  env: Env<'e>,
  /// Хендл для приема ошибок от нативных вызовов оракла. Позволяет затем получить код ошибки
  /// и ее описание.
  error: Handle<OCIError>,
}
impl<'e> Environment<'e> {
  /// Создает окружение -- менеджер подключений к базе данных. Параметр `mode` позволяет задать возможности,
  /// которые будут доступны при работе с базой данных.
  ///
  /// # OCI вызовы
  /// Осуществляет OCI вызов [`OCIEnvNlsCreate()`][new]. При разрушении объекта будет осуществлен OCI вызов
  /// [`OCITerminate()`][end].
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-0B6911A9-4B46-476C-BC5E-B87581666CD9
  /// [end]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-B7BC5F9E-811C-490A-B308-472A12D690D2
  pub fn new(mode: CreateMode) -> Result<Self> {
    let mut env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.new_error_handle());

    Ok(Environment { env: env, error: err })
  }
  /// Осуществляет подключение к базе данных с указанными параметрами.
  ///
  /// # OCI вызовы
  /// Осуществляет OCI вызов [`OCISessionBegin()`][new]. При разрушении объекта соединения будет осуществлен OCI вызов
  /// [`OCISessionEnd()`][end].
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-31B1FDB3-056E-4AF9-9B89-8DA6AA156947
  /// [end]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17123
  #[inline]
  pub fn connect<P: Into<ConnectParams>>(&'e self, params: P) -> Result<Connection<'e>> {
    Connection::new(&self, &params.into())
  }
  #[inline]
  fn new_handle<T: HandleType>(&self) -> Result<Handle<T>> {
    self.env.new_handle(self.error.native_mut())
  }
  #[inline]
  fn new_descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    Descriptor::new(&self)
  }
  /// Получает хендл для записи ошибок во время общения с базой данных. В случае возникновения ошибки при вызове
  /// FFI-функции она может быть получена из хендла с помощью вызова `decode(ffi_result)`.
  #[inline]
  fn error(&self) -> &Handle<OCIError> {
    &self.error
  }
}
//-------------------------------------------------------------------------------------------------
/// Представляет соединение к базе данных, с определенным пользователем и паролем.
/// Соединение зависит от окружения, создавшего его, таким образом, окружение является менеджером
/// соединений. При уничтожении окружения все соединения закрываются, а не закоммиченные транзакции
/// в них откатываются.
///
/// # OCI вызовы
/// Объект соединения создается последовательными OCI вызовами [`OCIServerAttach()`][new1] и [`OCISessionBegin()`][new2].
/// При разрушении объекта будет осуществлен сначала OCI вызов [`OCISessionEnd()`][end2], а затем [`OCIServerDetach()`][end1].
///
/// [new1]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-B6291228-DA2F-4CE9-870A-F94243141757
/// [end1]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17121
/// [new2]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-31B1FDB3-056E-4AF9-9B89-8DA6AA156947
/// [end2]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17123
#[derive(Debug)]
pub struct Connection<'e> {
  /// Хендл сервера, к которому будут направляться запросы. Несколько пользователей (подключений)
  /// могут одновременно работать с одним сервером через общий хендл. В настоящий момент это не
  /// поддерживается, каждое подключение использует свое сетевое соединение к серверу.
  server: Server<'e>,
  /// Хендл, хранящий информацию об учетных данных пользователя, независимо от того, к какой инстанции БД он
  /// подключен и подключен ли вообще.
  context: Handle<OCISvcCtx>,
  /// Хендл, хранящий информацию о логине конкретного пользователя БД к конкретному серверу БД.
  session: Handle<OCISession>,
  /// Режим аутетификации, который использовался при создании соединения. Необходим при закрытии
  auth_mode: AuthMode,
}
impl<'e> Connection<'e> {
  fn new(env: &'e Environment, params: &ConnectParams) -> Result<Self> {
    let server = try!(Server::new(env, Some(&params.dblink), params.attach_mode));
    let mut context: Handle<OCISvcCtx > = try!(env.new_handle());
    let mut session: Handle<OCISession> = try!(env.new_handle());

    let credMode = match params.credentials {
      Credentials::Rdbms { ref username, ref password } => {
        // Ассоциируем имя пользователя и пароль с сессией.
        // Надо отметить, что эти атрибуты сохраняются после закрытия сессии и при переподключении
        // можно их заново не устанавливать.
        try!(session.set_str(username, Attr::Username, &env.error));
        try!(session.set_str(password, Attr::Password, &env.error));

        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        CredentialMode::Rdbms
      },
      Credentials::Ext => CredentialMode::Ext,
    };

    // Ассоциируем сервер с контекстом и осуществляем подключение
    try!(context.set_handle(server.handle(), Attr::Server, &env.error));
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
    try!(context.set_handle(&session, Attr::Session, &env.error));

    Ok(Connection { server: server, context: context, session: session, auth_mode: params.auth_mode })
  }
  /// Получает хендл для записи ошибок во время общения с базой данных. Хендл берется из окружения, которое породило
  /// данное соединение. В случае возникновения ошибки при вызове FFI-функции она может быть получена из хендла с помощью
  /// вызова `decode(ffi_result)`.
  #[inline]
  fn error(&self) -> &Handle<OCIError> {
    self.server.error()
  }
  #[inline]
  unsafe fn as_descriptor<T: DescriptorType>(&self, raw: &[u8]) -> &T {
    let p = raw.as_ptr() as *const *const T;
    &*(*p as *const T)
  }

  /// Осуществляет разбор SQL-выражения и создает подготовленное выражение для дальнейшего эффективного исполнения запросов.
  /// Выражение использует родной для сервера базы данных синтаксис разбора запросов. Если вам требуется использовать конкретный
  /// синтаксис, воспользуйтесь методом [`prepare_with_syntax`][1].
  ///
  /// Полученное выражение не кешируется и повторный вызов данной функции с таким же текстом запроса приведет к запросу на сервер
  /// базы данных для разбора выражения.
  ///
  /// Возвращаемый объект выражения живет не дольше соединения, его породившего. Закрытия соединения автоматически закрывает все
  /// подготовленные выражения. Благодаря концепции времен жизни Rust не нужно беспокоится об этом, компилятор не позволит иметь
  /// ссылку на выражение, если соединение будет разрушено (если не использовать небезопасную `unsafe`-магию).
  ///
  /// # OCI вызовы
  /// Объект выражения создается OCI вызовом [`OCIStmtPrepare2()`][new]. При разрушении объекта соединения будет осуществлен
  /// OCI вызов [`OCIStmtRelease()`][end].
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17168
  /// [end]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17170
  ///
  /// [1]: #method.prepare_with_syntax
  #[inline]
  pub fn prepare(&'e self, sql: &str) -> Result<Statement<'e, 'e>> {
    self.prepare_with_syntax(Syntax::default(), sql)
  }
  /// Осуществляет разбор SQL-выражения и создает подготовленное выражение для дальнейшего эффективного исполнения запросов.
  /// При разборе текста выражения используется указанный синтаксис сервера базы данных. В большинстве случаев стоит предпочитать
  /// использование родного для базы данных синтаксиса разбора, так что рекомендуется использовать метод [`prepare`][1].
  ///
  /// Полученное выражение не кешируется и повторный вызов данной функции с таким же текстом запроса приведет к запросу на сервер
  /// базы данных для разбора выражения.
  ///
  /// Возвращаемый объект выражения живет не дольше соединения, его породившего. Закрытия соединения автоматически закрывает все
  /// подготовленные выражения. Благодаря концепции времен жизни Rust не нужно беспокоится об этом, компилятор не позволит иметь
  /// ссылку на выражение, если соединение будет разрушено (если не использовать небезопасную `unsafe`-магию).
  ///
  /// # OCI вызовы
  /// Объект выражения создается OCI вызовом [`OCIStmtPrepare2()`][new]. При разрушении объекта соединения будет осуществлен
  /// OCI вызов [`OCIStmtRelease()`][end].
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17168
  /// [end]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17170
  ///
  /// [1]: #method.prepare
  #[inline]
  pub fn prepare_with_syntax(&'e self, syntax: Syntax, sql: &str) -> Result<Statement<'e, 'e>> {
    Statement::new(&self, sql, None, syntax)
  }
}
impl<'e> Drop for Connection<'e> {
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

#[cfg(test)]
mod tests {
  use std::env;
  use super::*;
  use params::*;
  use types::*;
  #[test]
  fn it_works() {
    let env = Environment::new(CreateMode::default()).expect("Can't create ORACLE environment");

    let mut args = env::args();
    let _ = args.next().unwrap();// Путь к исходнику, запускаемому для тестов
    let _ = args.next();// Имя теста. приходится передавать, если есть строка подключения к базе

    let dblink = args.next().unwrap_or("".into());
    let cred = match args.next() {
      Some(username) => {
        Credentials::Rdbms {
          username: username,
          password: args.next().expect("Password must be set"),
        }
      },
      None => Credentials::Ext,
    };

    let params = ConnectParams {
      dblink: dblink,
      attach_mode: AttachMode::default(),
      credentials: cred,
      auth_mode: AuthMode::default(),
    };
    println!("params: {:?}", params);

    let conn = env.connect(params).expect("Can't connect to ORACLE database");
    let stmt = conn.prepare("select * from user_users").expect("Can't prepare statement");
    let rs = stmt.query().expect("Can't execute query");
    let columns = stmt.columns().expect("Can't get select list column count");
    for col in &columns {
      println!("col: {:?}", col);
    }
    println!("Now values:");
    for row in rs {
      let user: Result<Option<String>> = row.get(&columns[0], &conn);
      println!("row: user: {:?}", user);
    }
  }
}
