//! Содержит определение структуры для описания версии клиента и сервера и методы для ее преобразования в строку и разбора из строки.

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

pub use ffi::native::client_version;

/// Возможные ошибки разбора версии из строкового представления.
#[derive(Debug)]
pub enum ParseVersionError {
  /// Указанная часть версии не является целым неотрицательным числом в диапазоне `[0; i32::MAX]`.
  Part(u8, ParseIntError),
  /// Количество цифр версии, разделенных точкой, превышает 5 штук.
  Count,
}
/// Описывает версию клиента или сервера
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
  /// Старшая версия сервера или клиента. Для релиза `12.2с` это 12.
  pub major: i32,
  /// Младшая версия сервера или клиента. Для релиза `12.2с` это 2.
  pub minor: i32,
  /// The update number.
  pub update: i32,
  /// The patch number that was applied to the library.
  pub patch: i32,
  /// The port-specific patch applied to the library.
  pub port_update: i32,
}
impl Version {
  /// Формирует версию, в которой все поля, кроме [`major`](#structfield.major) равны `0`.
  #[inline]
  pub fn major(major: i32) -> Self { Self::minor(major, 0) }
  /// Формирует версию, в которой все поля, кроме [`major`](#structfield.major) и [`minor`](#structfield.minor) равны `0`.
  #[inline]
  pub fn minor(major: i32, minor: i32) -> Self { Self::update(major, minor, 0) }
  /// Формирует версию, в которой все поля, кроме [`major`](#structfield.major), [`minor`](#structfield.minor) и [`update`](#structfield.update) равны `0`.
  #[inline]
  pub fn update(major: i32, minor: i32, update: i32) -> Self { Self::patch(major, minor, update, 0) }
  /// Формирует версию, в которой заданы все поля, кроме поля [`port_update`](#structfield.port_update). Поле `port_update` равно `0`.
  #[inline]
  pub fn patch(major: i32, minor: i32, update: i32, patch: i32) -> Self {
    Version { major: major, minor: minor, update: update, patch: patch, port_update: 0 }
  }
}
impl Default for Version {
  /// Создает версию, в которой все поля равны `0`.
  fn default() -> Self {
    Version::major(0)
  }
}
impl fmt::Display for Version {
  /// Распечатывает версию в виде пяти чисел, разделенных точками. Из данного представления оно потом может быть распарсено
  /// при помощи типажа `FromStr`.
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "{}.{}.{}.{}.{}", self.major, self.minor, self.update, self.patch, self.port_update)
  }
}
impl FromStr for Version {
  type Err = ParseVersionError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut v = [0; 5];
    for (i, n) in s.split('.').map(i32::from_str).enumerate() {
      if i >= v.len() {
        return Err(ParseVersionError::Count);
      }
      if let Err(err) = n {
        return Err(ParseVersionError::Part(i as u8, err));
      }
      v[i] = n.unwrap();
    }
    Ok(Version { major: v[0], minor: v[1], update: v[2], patch: v[3], port_update: v[4] })
  }
}