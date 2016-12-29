#![feature(associated_consts)]
#![allow(non_snake_case)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;

pub mod error;
pub mod types;
mod ffi;
pub use ffi::*;

type Result<T> = std::result::Result<T, error::Error>;

/// Параметры подключения к базе данных
#[derive(Debug)]
pub struct ConnectParams {
  pub dblink: String,
  pub mode: AttachMode,
  /// Имя пользователя, под которым установить соединение к базе данных
  pub username: String,
  /// Пароль пользователя, под которым установить соединение к базе данных
  pub password: String,
}

#[cfg(test)]
mod tests {
  use std::env;
  use super::*;
  #[test]
  fn it_works() {
    let env = Environment::new(CreateMode::default()).expect("Can't create ORACLE environment");

    let mut args = env::args();
    let path = args.next().unwrap();
    let test_name = args.next();

    let params = ConnectParams {
      dblink: args.next().unwrap_or("".into()),
      mode: AttachMode::default(),
      // Скрипт настройки на трависе добавляет пользователя, из под которого запускается с пустым паролем
      username: args.next().unwrap_or_else(|| env::var("USER").expect("Environment variable USER not set")),
      password: args.next().unwrap_or("".into()),
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
