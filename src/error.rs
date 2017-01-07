//! Виды ошибок, которые могут генерироваться библиотекой.

use std::convert::From;
use std::error;
use std::fmt;

use types::Type;

/// Информация об одной ошибке/предупреждении Oracle
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Info {
  /// Код ошибки оракла, `ORA-xxxxx`.
  pub code: isize,
  /// Сообщение оракла об ошибке, полученной функцией `OCIErrorGet()`.
  pub message: String,
}

/// Ошибки, возникающие при вызове нативных функций Oracle,
#[derive(Debug)]
pub enum DbError {
  /// Функция выполнилась успешно, но есть диагностическая информация
  /// (функция вернула код `OCI_SUCCESS_WITH_INFO (==1)`).
  Info(Vec<Info>),
  /// Функция при своем выполнении исчерпала все данные, предоставленные ей и ей требуется еще
  /// (функция вернула код `OCI_NEED_DATA (==99)`).
  NeedData,
  /// Вызов функции получения данных не вернул никаких данных
  /// (функция вернула код `OCI_NO_DATA (==100)`).
  NoData,

  /// Ошибка вызова одной из функций API Oracle (функция вернула код `OCI_ERROR (==-1)`).
  /// Содержит код и сообщение об ошибке, полученное вызовом функции `OCIErrorGet()`.
  Fault(Info),
  /// Хендл, переданный в функцию, оказался некорректным
  /// (функция вернула код `OCI_INVALID_HANDLE (==-2)`).
  InvalidHandle,
  /// Возвращается из некоторых функций, в неблокирующем режиме, означает, что асинхронная операция
  /// начата, но еще не завершена (функция вернула код `OCI_STILL_EXECUTING (==-3123)`).
  StillExecuting,
  /// Функция вернула неизвестный код ошибки, не покрытый ни одним из предыдущих вариантов
  Unknown(isize),
}
impl fmt::Display for DbError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
impl error::Error for DbError {
  fn description(&self) -> &str {
    match *self {
      DbError::Info(_) => "Success execution, but diagnostic information present",
      DbError::NeedData => "Need additional data for continue execution",
      DbError::NoData => "No data",
      DbError::Fault(ref err) => &err.message,
      DbError::InvalidHandle => "Invalid handle passed to function",
      DbError::StillExecuting => "Asynchronous call of function not yet completed, still executing",
      DbError::Unknown(_) => "Unknown return code",
    }
  }
}
/// Ошибка, которую может вернуть библиотека. Включает ошибки взаимодействия с базой данных,
/// ошибки конвертации значений
#[derive(Debug)]
pub enum Error {
  /// Ошибка вызова одной из функций API Oracle.
  Db(DbError),
  /// Ошибка преобразования значения Rust в значение базы данных или наоборот
  Conversion(Type),
  /// Возникает при [получении элемента][get] из [строки выборки][row], если индекс, по которому получается элемент,
  /// не существует в выборке.
  ///
  /// [get]: ../stmt/struct.Row.html#method.get
  /// [row]: ../stmt/struct.Row.html
  InvalidColumn,
}
impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
impl error::Error for Error {
  fn description(&self) -> &str {
    match *self {
      Error::Db(ref err) => err.description(),
      Error::Conversion(_) => "Can't convert value from/to Rust to DB type",
      Error::InvalidColumn => "Nonexisting column",
    }
  }
  fn cause(&self) -> Option<&error::Error> {
    match *self {
      Error::Db(ref err) => Some(err),
      _ => None,
    }
  }
}
impl From<DbError> for Error {
  fn from(err: DbError) -> Self {
    Error::Db(err)
  }
}