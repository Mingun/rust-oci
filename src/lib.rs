#![feature(associated_consts)]
#![allow(non_snake_case)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;

pub mod error;
pub mod types;
mod ffi;
pub use ffi::*;

type Result<T> = std::result::Result<T, error::Error>;
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
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
  use std::env;
  use super::*;
  #[test]
  fn it_works() {
    let env = Environment::new(types::CreateMode::default()).expect("Can't create ORACLE environment");

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
      let user: Result<Option<&str>> = row.get(&columns[0]);
      println!("row: user: {:?}", user);
    }
  }
}
