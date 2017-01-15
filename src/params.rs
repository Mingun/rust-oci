//! Содержит структуры, описывающие параметры подключения к базе данных
use types::{AttachMode, AuthMode, Charset, CreateMode};

/// Параметры инициализации менеджера подключений к базе данных.
#[derive(Clone, Debug)]
pub struct InitParams {
  /// Возможности, которые будут доступны при работе с базой данных.
  pub mode: CreateMode,
  /// The client-side character set for the current environment handle. If it is 0, the `NLS_LANG`
  /// setting is used. `OCI_UTF16ID` is a valid setting; it is used by the metadata and the `CHAR` data.
  pub charset: Charset,
  /// The client-side national character set for the current environment handle. If it is `0`,
  /// `NLS_NCHAR` setting is used. `OCI_UTF16ID` is a valid setting; it is used by the `NCHAR` data.
  pub ncharset: Charset,
}
impl Default for InitParams {
  fn default() -> Self {
    InitParams { mode: Default::default(), charset: Default::default(), ncharset: Default::default() }
  }
}
impl Into<InitParams> for CreateMode {
  /// Преобразует режим инициализации менеджера подключений в парамтры менеджара подключений, в качетсве
  /// режима используя собственное значения и оставляя остальные параметры по умолчанию.
  fn into(self) -> InitParams {
    InitParams { mode: self, ..Default::default() }
  }
}
/// Содержит учетные данные пользователя, которые должны использоваться для аутентификации в базе.
#[derive(Clone, Debug)]
pub enum Credentials {
  /// База будет проводить аутентификацию по паре пользователь/пароль.
  Rdbms {
    /// Имя пользователя, под которым установить соединение к базе данных
    username: String,
    /// Пароль пользователя, под которым установить соединение к базе данных
    password: String,
  },
  /// База будет проводить аутентификацию, используя внешние учетные данные.
  /// Подключение всегда идет на локальной машине.
  Ext,
  //Proxy,
}
/// Параметры подключения к базе данных
#[derive(Clone, Debug)]
pub struct ConnectParams {
  /// Адрес базы и указатель сервиса, к которому следует подключиться.
  /// В случае внешней аутентификации не требуется, т.к. база всегда запущена на той же машине
  pub dblink: String,
  /// Режим создания соединений -- обычный или с использованием пула соединений.
  pub attach_mode: AttachMode,
  /// Учетные данные, используемые для логина в базу
  pub credentials: Credentials,
  /// Режим аутентификации, позволяющий задать дополнительные привелегии при подключении к базе данных.
  pub auth_mode: AuthMode,
}