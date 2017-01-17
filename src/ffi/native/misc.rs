//! Функции, описанные в разделе [Miscellaneous Functions][1] документации Oracle,
//! посвященном различным вспомогательным функциям работы с базой данных.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI167

use std::os::raw::{c_int, c_void, c_uchar, c_uint};

use DbResult;
use version::Version;

use ffi::Handle;// Основные типобезопасные примитивы
use ffi::{VersionHandle, InterruptHandle};// Типажи для безопасного моста к FFI

use ffi::native::OCIError;// FFI типы

/// Получает версию клиентской библиотеки. Для получения версии сервера необходимо [установить с ним соединение][1] и
/// воспользоваться вызовом [`Connection::server_version()`][2].
///
/// # Запросы к серверу (0)
/// Данный вызов не требует общения с сервером.
///
/// [1]: ../struct.Environment.html#method.connect
/// [2]: ../struct.Connection.html#method.server_version
pub fn client_version() -> Version {
  let mut ver = Version::default();
  unsafe {
    OCIClientVersion(
      &mut ver.major,
      &mut ver.minor,
      &mut ver.update,
      &mut ver.patch,
      &mut ver.port_update
    )
  };
  return ver;
}
/// Возвращает версию сервера Oracle-а, к которому подключен клиент.
///
/// # Параметры
/// - hndl:
///   Хендл, из которого можно получить версию сервера.
/// - err:
///   Хендл для сбора ошибок, из которого будут извлечены подробности ошибки в случае, если она произойдет.
///
/// # Запросы к серверу (1)
/// Функция выполняет один запрос к серверу при каждом вызове.
pub fn server_version<T: VersionHandle>(hndl: &Handle<T>, err: &Handle<OCIError>) -> DbResult<Version> {
  let mut ver = 0;
  // Необходимо передать в функцию массив с длиной хотя бы в 1 элемент, иначе программа падает с Segmentation Fault.
  // Чтобы не выделять память, можно просто передать 1 байт в одной переменной.
  let mut buf = 0;
  let res = unsafe {
    OCIServerRelease(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      &mut buf, 1,
      T::ID as c_uchar,
      &mut ver
    )
  };
  match res {
    0 => Ok(to_version(ver)),
    e => Err(err.decode(e)),
  }
}
/// Распаковывает число с версией сервера, которую вернул OCI вызов в структуру с версией
#[inline]
fn to_version(v: c_uint) -> Version {
  Version {
    major:  ((v >> 24) & 0x000000FF) as i32,
    minor:  ((v >> 20) & 0x0000000F) as i32,
    update: ((v >> 12) & 0x000000FF) as i32,
    patch:  ((v >>  8) & 0x0000000F) as i32,
    port_update: (v    & 0x000000FF) as i32,
  }
}
/// Позволяет отменить слишком долго выполняющийся запрос к серверу. Также требуется вызывать для прекращения чтения LOB-а.
///
/// # Параметры
/// - hndl:
///   Хендл, из которого можно получить версию сервера.
/// - err:
///   Хендл для сбора ошибок, из которого будут извлечены подробности ошибки в случае, если она произойдет.
///
/// # OCI вызовы
/// Функция вызывает [`OCIBreak()`][1] для прекращения ожидания асинхронной операции или операции чтения по частям (piecewice).
///
/// # Запросы к серверу (1)
/// Функция выполняет один запрос к серверу при каждом вызове.
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17285
pub fn break_<T: InterruptHandle>(hndl: &Handle<T>, err: &Handle<OCIError>) -> DbResult<()> {
  let res = unsafe {
    OCIBreak(
      hndl.native_mut() as *mut c_void,
      err.native_mut()
    )
  };
  err.check(res)
}
/// Восстанавливает прерванную асинхронную операцию и протокол. Должна вызываться только в неблокирующем режиме и только после того,
/// как был вызван [`break_()`][].
///
/// # Параметры
/// - hndl:
///   Хендл, из которого можно получить версию сервера.
/// - err:
///   Хендл для сбора ошибок, из которого будут извлечены подробности ошибки в случае, если она произойдет.
///
/// # OCI вызовы
/// Функция вызывает [`OCIReset()`][2] для прекращения ожидания асинхронной операции или операции чтения по частям (piecewice).
///
/// # Запросы к серверу (0)
/// Функция не выполняет запросов к серверу.
///
/// [1]: #function.break_
/// [2]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17291
pub fn reset<T: InterruptHandle>(hndl: &Handle<T>, err: &Handle<OCIError>) -> DbResult<()> {
  let res = unsafe {
    OCIReset(
      hndl.native_mut() as *mut c_void,
      err.native_mut()
    )
  };
  err.check(res)
}
// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Returns an error message in the buffer provided and an Oracle Database error code.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17287
  pub fn OCIErrorGet(hndlp: *mut c_void,
                     recordno: c_uint,
                     sqlstate: *mut c_uchar,// устарел с версии 8.x
                     errcodep: *mut c_int,  // возвращаемый код ошибки
                     bufp: *mut c_uchar,    // возвращаемое сообщение об ошибке
                     bufsiz: c_uint,
                     htype: c_uint) -> c_int;

  /// Returns the 5 digit Oracle Database version number of the client library at run time.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17286
  fn OCIClientVersion(major_version: *mut c_int,
                      minor_version: *mut c_int,
                      update_num: *mut c_int,
                      patch_num: *mut c_int,
                      port_update_num: *mut c_int);
  /// Returns the Oracle Database release string.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17293
  fn OCIServerRelease(hndlp: *mut c_void,
                      errhp: *mut OCIError,
                      bufp: *mut c_uchar,
                      bufsz: c_uint,
                      hndltype: c_uchar,
                      version: *mut c_uint) -> c_int;

  /// Performs an immediate (asynchronous) termination of any currently executing OCI function that is associated with a server.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17285
  fn OCIBreak(hndlp: *mut c_void,
              errhp: *mut OCIError) -> c_int;

  /// Resets the interrupted asynchronous operation and protocol. Must be called if an `OCIBreak()` call was issued while a
  /// nonblocking operation was in progress.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17291
  fn OCIReset(hndlp: *mut c_void,
              errhp: *mut OCIError) -> c_int;
}