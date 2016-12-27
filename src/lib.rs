#![feature(associated_consts)]
#![allow(non_snake_case)]
// Для типажей числовых типов, чтобы можно было реализовать управление атрибутами в обобщенном виде
extern crate num_integer;

mod ffi;
pub use ffi::*;

#[derive(Debug)]
pub struct Error {
  /// Результат вызова функции, которая вернула ошибку.
  pub result: isize,
  /// Код ошибки оракла, `ORA-xxxxx`.
  pub code: isize,
  /// Сообщение оракла об ошибке, полученной функцией `OCIErrorGet`.
  pub message: String,
}
impl Error {
  fn unknown(result: isize) -> Self {
    Error { result: result, code: 0, message: String::new() }
  }
}
type Result<T> = std::result::Result<T, Error>;

/// Параметры подключения к базе данных
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
    let params = ConnectParams {
      dblink: args.next().unwrap_or("".into()),
      mode: AttachMode::default(),
      // Скрипт настройки на трависе добавляет пользователя, из под которого запускается с пустым паролем
      username: args.next().unwrap_or_else(|| env::var("USER").expect("Environment variable USER not set")),
      password: args.next().unwrap_or("".into()),
    };
    let conn = env.connect(params).expect("Can't connect to ORACLE database");
    let stmt = conn.prepare("select * from dba_users").expect("Can't prepare statement");
    let rs = stmt.query().expect("Can't execute query");
    for col in stmt.columns().expect("Can't get select list column count") {
      println!("param: {:?}", col);
    }
    println!("Now values:");
    for d in rs {
      println!("value: {:?}", d);
    }
  }
}
