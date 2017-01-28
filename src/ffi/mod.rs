
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use DbResult;
use error::{DbError, Info};
use self::native::OCIErrorGet;// FFI функции

pub mod attr;
pub mod types;
pub mod native;

mod env;
mod handle;
mod descriptor;
mod server;

pub use self::env::Env;
pub use self::server::Server;
pub use self::handle::Handle;
pub use self::descriptor::{Descriptor, GenericDescriptor};

/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIHandleAlloc`][new] для создания хендла.
/// Ассоциированная константа `ID` указывает тип хендла, который будет передан в функцию.
///
/// [new]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#GUID-C5BF55F7-A110-4CB5-9663-5056590F12B5
pub trait HandleType {
  /// Вид хендла, используемый типом, реализующим данный типаж.
  const ID: types::Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIDescriptorAlloc`][new] для создания дескриптора.
/// Ассоциированная константа `ID` указывает тип дескриптора, который будет передан в функцию.
///
/// [new]: http://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17132
pub trait DescriptorType {
  /// Вид дескриптора, используемый типом, реализующим данный типаж.
  const ID: types::Descriptor;
}
//-------------------------------------------------------------------------------------------------
/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIErrorGet`][1] для получения информации об ошибке.
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17287
pub trait ErrorHandle {
  /// Тип хендла, из которого получается описание ошибки. Последний параметр функции [`OCIErrorGet`][1] (`type`).
  ///
  /// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17287
  const ID: types::Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функции [`OCIAttrGet`][get]/[`OCIAttrSet`][set]
/// для получения или установки атрибута.
///
/// [get]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17130
/// [set]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17131
pub trait AttrHandle {
  /// Тип хендла, владеющего атрибутами. Второй аргумент функций [`OCIAttrGet`][get]/[`OCIAttrSet`][set] (`trghndltyp`).
  ///
  /// [get]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17130
  /// [set]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17131
  const ID: types::Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функции [`OCIParamGet`][get]/[`OCIParamSet`][set]
/// для получения информации о параметре.
///
/// [get]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17136
/// [set]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17137
pub trait ParamHandle {
  /// Тип хендла, владеющего параметрами. Второй аргумент функций [`OCIParamGet`][get]/[`OCIParamSet`][set] (`htype`).
  ///
  /// [get]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17136
  /// [set]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17137
  const ID: types::Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функции [`OCIServerRelease`][1]/[`OCIServerVersion`][2]
/// для получения информации о версии сервера Oracle. Ассоциированная константа [`ID`][id] передается
/// предпоследним/последним параметром в эти функции (`hndltype`).
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17293
/// [2]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17294
/// [id]: ./trait.HandleType.html#associatedconstant.ID
pub trait VersionHandle : HandleType {}

/// Тип, реализующий данный типаж, может быть передан в функции [`OCIBreak`][1]/[`OCIReset`][2] для прекращения
/// выполнения асинхронной операции и восстановления состояния после такого прекращения соответственно.
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17285
/// [2]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17291
pub trait InterruptHandle : HandleType {}

//-------------------------------------------------------------------------------------------------
/// Транслирует результат, возвращенный любой функцией, в код ошибки базы данных
///
/// # Параметры
/// - handle:
///   Хендл, и которого можно извлечь информацию об ошибке. Обычно это специальный хендл `OCIError`, но
///   в тех случаях, когда его нет (создание этого хендла ошибки и, почему-то, окружения), можно использовать
///   хендл окружения `OCIEnv`
/// - error_no:
///   Вызовы функций могут возвращать множество ошибок. Это получаемый номер ошибки (нумерация с 1)
/// - msg:
///   Буфер, куда будет записано сообщение оракла об ошибке
fn decode_error_piece<T: ErrorHandle>(handle: *mut T, error_no: u32) -> (c_int, Info) {
  let mut code: c_int = 0;
  // Сообщение получается в кодировке, которую установили для хендла окружения.
  // Оракл рекомендует использовать буфер величиной 3072 байта
  let mut buf: Vec<u8> = Vec::with_capacity(3072);
  let res = unsafe {
    OCIErrorGet(
      handle as *mut c_void,
      error_no,
      ptr::null_mut(),// Устаревший с версии 8.x параметр, не используется
      &mut code,
      buf.as_mut_ptr(),
      buf.capacity() as u32,
      T::ID as u32
    )
  };
  // 100 == NoData - больше нет данных для расшифровки. В буфере может записаться мусор, поэтому не используем его
  if res == 100 {
    return (res, Info { code: code as isize, message: String::with_capacity(0) });
  }
  unsafe {
    // Так как функция только заполняет массив, но не возвращает длину, ее нужно вычислить и задать,
    // иначе трансформация в строку ничего не даст, т.к. будет считать массив пустым.
    let msg = CStr::from_ptr(buf.as_ptr() as *const c_char);
    buf.set_len(msg.to_bytes().len());
  };

  (res, Info { code: code as isize, message: String::from_utf8(buf).expect("Invalid UTF-8 from OCIErrorGet") })
}
fn decode_error_full<T: ErrorHandle>(handle: *mut T) -> Vec<Info> {
  let mut vec = Vec::new();

  for i in 1.. {
    let (res, info) = decode_error_piece(handle, i);
    if res == 100 {// 100 == NoData
      break;
    }
    vec.push(info)
  }
  return vec;
}
fn decode_error<T: ErrorHandle>(handle: *mut T, result: c_int) -> DbError {
  match result {
    // Относительный успех
    0 => unreachable!(),// Сюда не должны попадать
    1 => DbError::Info(decode_error_full(handle)),
    99 => DbError::NeedData,
    100 => DbError::NoData,

    // Ошибки
    -1 => {
      DbError::Fault(decode_error_piece(handle, 1).1)
    },
    -2 => DbError::InvalidHandle,
    -3123 => DbError::StillExecuting,
    e => DbError::Unknown(e as isize),
  }
}

/// Проверяет результат вызова FFI функции и возвращает либо успех в случае, если результат равен `0`,
/// либо [неизвестную ошибку][1] базы данных. Используется в случаях, когда необходимо преобразовать
/// ошибку базы данных в тип Rust, но хендла ошибки еще или уже нет.
///
/// [1]: ../error/enum.DbError.html#variant.Unknown
fn check(native: c_int) -> DbResult<()> {
  return match native {
    0 => Ok(()),
    e => Err(DbError::Unknown(e as isize))
  };
}