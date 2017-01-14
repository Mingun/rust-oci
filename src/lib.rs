//! Биндинг Oracle Call Interface для Rust
//! --------------------------------------
//! [![Build Status](https://travis-ci.org/Mingun/rust-oci.svg?branch=master)](https://travis-ci.org/Mingun/rust-oci)
//!
//! # Пример использования
//! ```rust
//! extern crate oci;
//! use oci::Environment;
//! use oci::params::{ConnectParams, Credentials};
//! use oci::types::{AttachMode, AuthMode, CreateMode};
//! use oci::version::client_version;
//! 
//! fn main() {
//!   // Инициализируем клиентскую библиотеку Oracle
//!   let env = Environment::new(CreateMode::default()).expect("Can't create ORACLE environment");
//! 
//!   // Создаем параметры. Вскоре их можно будет распарсить из строки (jdbc и sql*plus версий)
//!   let params = ConnectParams {
//!     dblink: "".into(),
//!     attach_mode: AttachMode::default(),
//!     // Учетные данные, в данном случае аутентификация по паролю.
//!     credentials: Credentials::Rdbms { username: "username".into(), password: "password".into() },
//!     auth_mode: AuthMode::default(),
//!   };
//! 
//!   // Соединяемся с сервером
//!   let conn = env.connect(params).expect("Can't connect to ORACLE database");
//!   println!("Client version: {}", client_version());
//!   println!("Server version: {}", conn.server_version().expect("Can't get server version"));
//!   println!("Connection time offset: {:?}", conn.get_current_time_offset().expect("Can't get connection time offset"));
//! 
//!   // Готовим запрос для выполнения
//!   let mut stmt = conn.prepare("select * from user_users").expect("Can't prepare statement");
//! 
//!   // Выполняем! Bind-параметры пока не поддерживаются
//!   let rs = stmt.query().expect("Can't execute query");
//!   // ...продолжение следует...
//! }
//! ```

#![feature(associated_consts)]
#![allow(non_snake_case)]
#![deny(missing_docs)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;
extern crate num_traits;

pub mod convert;
pub mod error;
pub mod lob;
pub mod params;
pub mod stmt;
pub mod types;
pub mod version;
mod ffi;

/// Тип результата, возвращаемый всеми функциями библиотеки, которые могут привести к ошибке.
/// В большинстве случаев библиотека никогда не генерирует панику, всегда возвращая ошибочный
/// результат в виде ошибке. Немногочисленные исключения документированы особо, и существуют
/// потому, что внешнее по отношению к библиотеке API не позволяет вернуть ошибку (Например,
/// из реализации типажа [`Drop`][1]).
///
/// [1]: https://doc.rust-lang.org/std/ops/trait.Drop.html
pub type Result<T> = std::result::Result<T, error::Error>;
/// Тип результата, возвращаемый функциями-обертками, непосредственно вызывающие с OCI функции
/// через FFI интерфейс.
type DbResult<T> = std::result::Result<T, error::DbError>;

use std::os::raw::c_uint;

use params::{ConnectParams, Credentials};
use stmt::Statement;
use types::{CreateMode, AuthMode, Syntax};
use version::Version;

use ffi::{Env, Server, Handle, Descriptor};// Основные типобезопасные примитивы
use ffi::{HandleType, DescriptorType};// Типажи для безопасного моста к FFI

use ffi::types::{Attr, CredentialMode};
use ffi::native::{OCIEnv, OCISvcCtx, OCISession, OCIError};// FFI типы
use ffi::native::{OCISessionBegin, OCISessionEnd};// FFI функции
use ffi::native::time::{get_time_offset, sys_timestamp, TimestampWithTZ};

// Для того, чтобы пользоваться функциями типажей, они должны быть в области видимости
use ffi::attr::AttrHolder;

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
  /// # Запросы к серверу (1)
  /// Функция выполняет один запрос к серверу при создании каждого соединения. Также будет совершен один запрос к серверу
  /// при уничтожении соединения.
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#GUID-31B1FDB3-056E-4AF9-9B89-8DA6AA156947
  /// [end]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17123
  #[inline]
  pub fn connect<P: Into<ConnectParams>>(&'e self, params: P) -> Result<Connection<'e>> {
    Connection::new(&self, &params.into()).map_err(Into::into)
  }
  /// Создает новый хендл для хранения объектов указанного типа. Хендл будет автоматически закрыт при выходе из зоны видимости
  /// переменной, хранящей его.
  #[inline]
  fn new_handle<T: HandleType>(&self) -> DbResult<Handle<T>> {
    self.env.new_handle(self.error.native_mut())
  }
  /// Создает новый дескриптор для хранения объектов указанного типа. Дескриптор будет автоматически закрыт при выходе из зоны
  /// видимости переменной, хранящей его.
  #[inline]
  fn new_descriptor<T: DescriptorType>(&self) -> DbResult<Descriptor<T>> {
    Descriptor::new(&self)
  }
  /// Получает хендл для записи ошибок во время общения с базой данных. В случае возникновения ошибки при вызове
  /// FFI-функции она может быть получена из хендла с помощью вызова `decode(ffi_result)`.
  #[inline]
  fn error(&self) -> &Handle<OCIError> {
    &self.error
  }
  /// Получает голый указатель на хендл окружения, используемый для передачи в нативные функции.
  #[inline]
  fn native(&self) -> *const OCIEnv {
    self.env.native()
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
  fn new(env: &'e Environment, params: &ConnectParams) -> DbResult<Self> {
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

  /// Получает окружение, которое создало данное соединение.
  #[inline]
  pub fn get_env(&self) -> &'e Environment<'e> {
    self.server.get_env()
  }
  /// Возвращает версию сервера Oracle-а, к которому подключен клиент.
  ///
  /// Данная функция возвращает версию сервера. Если вам нужно получить версию клиента, то используйте вызов [`client_version()`][1].
  ///
  /// # OCI вызовы
  /// Для получения версии сервера используется OCI вызов [`OCIServerRelease()`][2].
  ///
  /// # Запросы к серверу (1)
  /// Функция выполняет один запрос к серверу при каждом вызове.
  ///
  /// [1]: ./fn.client_version.html
  /// [2]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17293
  pub fn server_version(&self) -> Result<Version> {
    self.server.version().map_err(Into::into)
  }
  /// Осуществляет разбор SQL-выражения и создает подготовленное выражение для дальнейшего эффективного исполнения запросов.
  /// Выражение использует родной для сервера базы данных синтаксис разбора запросов. Если вам требуется использовать конкретный
  /// синтаксис, воспользуйтесь методом [`prepare_with_syntax`][1].
  ///
  /// Полученное выражение не кешируется и повторный вызов данной функции с таким же текстом запроса приведет к запросу на сервер
  /// базы данных для разбора выражения.
  ///
  /// Возвращаемый объект выражения живет не дольше соединения, его породившего. Закрытие соединения автоматически закрывает все
  /// подготовленные выражения. Благодаря концепции времен жизни Rust не нужно беспокоиться об этом, компилятор не позволит иметь
  /// ссылку на выражение, если соединение будет разрушено (если не использовать небезопасную `unsafe`-магию).
  ///
  /// # OCI вызовы
  /// Объект выражения создается OCI вызовом [`OCIStmtPrepare2()`][new]. При разрушении объекта соединения будет осуществлен
  /// OCI вызов [`OCIStmtRelease()`][end].
  ///
  /// # Запросы к серверу (0)
  /// Функция не выполняет запросов к серверу, разбор и подготовка запроса выполняются локально.
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
  /// Возвращаемый объект выражения живет не дольше соединения, его породившего. Закрытие соединения автоматически закрывает все
  /// подготовленные выражения. Благодаря концепции времен жизни Rust не нужно беспокоиться об этом, компилятор не позволит иметь
  /// ссылку на выражение, если соединение будет разрушено (если не использовать небезопасную `unsafe`-магию).
  ///
  /// # OCI вызовы
  /// Объект выражения создается OCI вызовом [`OCIStmtPrepare2()`][new]. При разрушении объекта соединения будет осуществлен
  /// OCI вызов [`OCIStmtRelease()`][end].
  ///
  /// # Запросы к серверу (0)
  /// Функция не выполняет запросов к серверу, разбор и подготовка запроса выполняются локально.
  ///
  /// [new]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17168
  /// [end]: http://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI17170
  ///
  /// [1]: #method.prepare
  #[inline]
  pub fn prepare_with_syntax(&'e self, syntax: Syntax, sql: &str) -> Result<Statement<'e, 'e>> {
    Statement::new(&self, sql, None, syntax).map_err(Into::into)
  }
  /// Получает текущий часовой пояс сессии в виде пары чисел, означающих смещение в часах и минутах.
  /// Диапазон возможных значений результата: от `-12:59` до `+14:00`.
  ///
  /// Данный часовой пояс влияет на получение дат из столбцов с типом [`TIMESTAMP WITH LOCAL TIME ZONE`][tz] при извлечении их в
  /// Rust-тип [`chrono::NaiveDate`][nd], [`chrono::NaiveTime`][nt] и [`chrono::NaiveDateTime`][ndt] (конвертация в данные типы
  /// доступна только в том случае, если библиотека используется с возможностью `with-chrono`).
  ///
  /// # OCI вызовы
  /// Напрямую получить часовой пояс, аналогично запросу `select sessionTimeZone from dual` нельзя, API не предоставляет такой функции,
  /// поэтому смещение извлекается из текущего времени сервера базы данных, запрашиваемого OCI-вызовом [`OCIDateTimeSysTimeStamp()`][1].
  /// Часовой пояс затем извлекается из полученного объекта временной метки вызовом [`OCIDateTimeGetTimeZoneOffset()`][2]. Кроме того,
  /// для возможности извлечь время создается и уничтожается дескриптор для временной метки с часовым поясом, соответственно функциями
  /// [`OCIDescriptorAlloc()`][new] и [`OCIDescriptorFree()`][end].
  ///
  /// # Запросы к серверу (0)
  /// Ни одна из вызываемых функций не выполняет запросов к серверу.
  ///
  /// [tz]: http://docs.oracle.com/database/122/LNOCI/data-types.htm#LNOCI16308
  /// [nd]: https://lifthrasiir.github.io/rust-chrono/chrono/naive/date/struct.NaiveDate.html
  /// [nt]: https://lifthrasiir.github.io/rust-chrono/chrono/naive/time/struct.NaiveTime.html
  /// [ndt]: https://lifthrasiir.github.io/rust-chrono/chrono/naive/datetime/struct.NaiveDateTime.html
  /// [1]: http://docs.oracle.com/database/122/LNOCI/oci-date-datetime-and-interval-functions.htm#LNOCI17425
  /// [2]: http://docs.oracle.com/database/122/LNOCI/oci-date-datetime-and-interval-functions.htm#LNOCI17422
  /// [new]: http://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17132
  /// [end]: http://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17134
  pub fn get_current_time_offset(&self) -> Result<(i8, i8)> {
    let mut d: Descriptor<TimestampWithTZ> = try!(self.server.new_descriptor());
    // Получаем текущее время сервера. Так как оно возвращается в виде временной метки с часовым поясом,
    // а часовой пояс зависит от часового пояса сессии, то таким образом мы можем получить часовой пояс сессии.
    try!(sys_timestamp(&self.session, self.error(), d.native_mut()));
    get_time_offset(&self.session, self.error(), d.as_ref()).map_err(Into::into)
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

/// Типаж, предоставляющий классу соединения возможность создавать выражения, при этом не выставляя данную возможность
/// в виде публичного API соединения.
trait StatementPrivate {
  /// Создает новый объект подготовленного выражения для указанного соединения. Осуществляет разбор переданного выражения,
  /// при необходимости может поискать его в кеше уже подготовленных выражений.
  ///
  /// # Параметры
  /// - conn:
  ///   Соединение, создающее выражение. Полученное выражение будет жить не дольше него.
  /// - sql:
  ///   Текстовое представление запроса
  /// - key:
  ///   Если задано, то выражение будет искаться в кеше выражений по указанному ключу и если оно будет найдено, повторного
  ///   синтаксического анализа производится не будет. В этом случае параметр `syntax` не учитывается.
  /// - syntax:
  ///   Правила разбора, которые будет использоваться при анализе SQL-выражения.
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: Syntax) -> DbResult<Statement<'c, 'k>>;
}

#[cfg(test)]
mod tests {
  #[cfg(feature = "with-chrono")]
  extern crate chrono;

  use std::env;
  use super::*;
  use convert::*;
  use params::*;
  use stmt::*;
  use types::*;
  use version::client_version;

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
    println!("Client version: {}", client_version());
    println!("Server version: {}", conn.server_version().expect("Can't get server version"));
    println!("Connection time offset: {:?}", conn.get_current_time_offset().expect("Can't get connection time offset"));

    let mut stmt = conn.prepare("select * from user_users").expect("Can't prepare statement");
    {
      let rs = stmt.query().expect("Can't execute query");
      for col in rs.columns() {
        println!("col: {:?}", col);
      }

      println!("Now values:");
      for row in &rs {
        let user: Result<Option<String>> = row.get(0);
        println!("row: user: {:?}", user);
      }
    }

    if cfg!(feature = "with-chrono") {
      println!("Connection time offset: {:?}", conn.get_current_time_offset().expect("Can't get connection time offset"));
      print_chrono(&mut stmt);

      println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
      conn.prepare("alter session set time_zone = '+00:00'").expect("set timezone").execute().expect("execute");

      println!("Connection time offset: {:?}", conn.get_current_time_offset().expect("Can't get connection time offset"));
      print_chrono(&mut stmt);
    }
  }
  #[cfg(not(feature = "with-chrono"))]
  fn print_chrono(_: &mut Statement) {}
  #[cfg(feature = "with-chrono")]
  fn print_chrono(stmt: &mut Statement) {
    let rs = stmt.query().expect("Can't execute query");
    let columns = rs.columns();
    for row in &rs {
      println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Naive");
      print_naive(&row, &columns[3]);// Timestamp
      print_naive(&row, &columns[7]);// TimestampWithTZ
      print_naive(&row, &columns[8]);// TimestampWithLTZ

      println!("------------------------------------------------------ FixedOffset");
      print_with_tz::<chrono::FixedOffset>(&row, &columns[3]);// Timestamp
      print_with_tz::<chrono::FixedOffset>(&row, &columns[7]);// TimestampWithTZ
      print_with_tz::<chrono::FixedOffset>(&row, &columns[8]);// TimestampWithLTZ

      println!("====================================================== UTC");
      print_with_tz::<chrono::UTC>(&row, &columns[3]);// Timestamp
      print_with_tz::<chrono::UTC>(&row, &columns[7]);// TimestampWithTZ
      print_with_tz::<chrono::UTC>(&row, &columns[8]);// TimestampWithLTZ
    }
  }
  #[cfg(feature = "with-chrono")]
  fn print_naive(row: &Row, col: &Column) {
    let time0: Result<Option<chrono::NaiveDate>> = row.get(col.pos);
    let time1: Result<Option<chrono::NaiveTime>> = row.get(col.pos);
    let time2: Result<Option<chrono::NaiveDateTime>> = row.get(col.pos);

    println!("column: {:?}", col);
    println!("chrono::date: {:?}", time0);
    println!("chrono::time: {:?}", time1);
    println!("chrono::datetime: {:?}", time2);
    println!();
  }
  #[cfg(feature = "with-chrono")]
  fn print_with_tz<'conn, Tz>(row: &Row<'conn>, col: &Column)
    where Tz: chrono::TimeZone,
          chrono::Date<Tz>: FromDB<'conn>,
          chrono::DateTime<Tz>: FromDB<'conn>
  {
    let time0: Result<Option<chrono::Date<Tz>>> = row.get(col.pos);
    let time2: Result<Option<chrono::DateTime<Tz>>> = row.get(col.pos);

    println!("column: {:?}", col);
    println!("chrono::date: {:?}", time0);
    println!("chrono::datetime: {:?}", time2);
    println!();
  }
}
