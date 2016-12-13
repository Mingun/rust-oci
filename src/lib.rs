#![feature(associated_consts)]
#![allow(non_snake_case)]

mod ffi;
pub use ffi::*;

#[derive(Debug)]
pub struct Error(std::os::raw::c_int);
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
  #[test]
  fn it_works() {
    
  }
}
