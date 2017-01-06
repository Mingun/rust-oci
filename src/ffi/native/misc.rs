//! Функции, описанные в разделе [Miscellaneous Functions][1] документации Oracle,
//! посвященном различным вспомогательным функциям работы с базой данных.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI167

use std::os::raw::{c_int, c_void, c_uchar, c_uint};

use Result;
use version::Version;

use ffi::Handle;// Основные типобезопасные примитивы
use ffi::VersionHandle;// Типажи для безопасного моста к FFI

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
pub fn server_version<T: VersionHandle>(hndl: &Handle<T>, err: &Handle<OCIError>) -> Result<Version> {
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
// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Returns an error message in the buffer provided and an Oracle Database error code.
  pub fn OCIErrorGet(hndlp: *mut c_void,
                     recordno: c_uint,
                     sqlstate: *mut c_uchar,// устарел с версии 8.x
                     errcodep: *mut c_int,  // возвращаемый код ошибки
                     bufp: *mut c_uchar,    // возвращаемое сообщение об ошибке
                     bufsiz: c_uint,
                     htype: c_uint) -> c_int;

  fn OCIClientVersion(major_version: *mut c_int,
                      minor_version: *mut c_int,
                      update_num: *mut c_int,
                      patch_num: *mut c_int,
                      port_update_num: *mut c_int);
  /// Returns the Oracle Database release string.
  /// 
  /// # Parameters
  /// - hndlp (IN):
  ///   The service context handle or the server context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - bufp (IN/OUT):
  ///   The buffer in which the release string is returned.
  /// - bufsz (IN):
  ///   The length of the buffer in number of bytes.
  /// - hndltype (IN):
  ///   The type of handle passed to the function.
  /// - version (IN/OUT):
  ///   The release string in integer format.
  /// 
  /// # Comments
  /// The buffer pointer `bufp` points to the release information in a string representation up to the `bufsz` including the `NULL` terminator.
  /// If the buffer size is too small, the result is truncated to the size `bufsz`. The version argument contains the 5-digit Oracle Database
  /// release string in integer format, which can be retrieved using the following macros:
  /// ```c,no_run
  /// #define MAJOR_NUMVSN(v) ((sword)(((v) >> 24) & 0x000000FF))      /* version number */ 
  /// #define MINOR_NUMRLS(v) ((sword)(((v) >> 20) & 0x0000000F))      /* release number */
  /// #define UPDATE_NUMUPD(v) ((sword)(((v) >> 12) & 0x000000FF))     /* update number */ 
  /// #define PORT_REL_NUMPRL(v) ((sword)(((v) >> 8) & 0x0000000F))    /* port release number */ 
  /// #define PORT_UPDATE_NUMPUP(v) ((sword)(((v) >> 0) & 0x000000FF)) /* port update number */
  /// ```
  fn OCIServerRelease(hndlp: *mut c_void,
                      errhp: *mut OCIError,
                      bufp: *mut c_uchar,
                      bufsz: c_uint,
                      hndltype: c_uchar,
                      version: *mut c_uint) -> c_int;
}