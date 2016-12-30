#![feature(associated_consts)]
#![allow(non_snake_case)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;

pub mod error;
pub mod types;
mod ffi;
pub use ffi::*;

type Result<T> = std::result::Result<T, error::Error>;

/// Содержит учетные данные пользователя, которые должны использоваться для аутентификации в базе.
#[derive(Clone, Debug)]
pub enum Credentials {
  /// База будет проводить аутентификацию по паре пользователь/пароль.
  Rdbms {
    /// Адрес базы и указатель сервиса, к которому следует подключиться.
    /// В случае внешней аутентификации не требуется, т.к. база всегда запущена на той же машине
    dblink: String,
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
  pub mode: AttachMode,
  /// Учетные данные, используемые для логина в базу
  pub credentials: Credentials,
}

#[cfg(test)]
mod tests {
  use std::env;
  use super::*;
  #[test]
  fn it_works() {
    let env = Environment::new(CreateMode::default()).expect("Can't create ORACLE environment");

    let mut args = env::args();
    let _ = args.next().unwrap();// Путь к исходнику, запускаемому для тестов
    let _ = args.next();// Имя теста. приходится передавать, если есть строка подключения к базе

    let cred = match args.next() {
      Some(dblink) => {
        Credentials::Rdbms {
          dblink: dblink,
          // Скрипт настройки на трависе добавляет пользователя, из под которого запускается с пустым паролем
          username: args.next().unwrap_or_else(|| env::var("USER").expect("Environment variable USER not set")),
          password: args.next().unwrap_or("".into()),
        }
      },
      None => Credentials::Ext,
    };

    let params = ConnectParams {
      mode: AttachMode::default(),
      credentials: cred,
    };
    println!("params: {:?}", params);

    let conn = env.connect(params).expect("Can't connect to ORACLE database");
    let stmt = conn.prepare("select * from dba_users").expect("Can't prepare statement");
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
