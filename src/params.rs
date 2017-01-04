//! Содержит структуры, описывающие параметры подключения к базе данных
use types::{AttachMode, AuthMode};

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