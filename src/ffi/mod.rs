
use std::os::raw::c_int;

use Result;
use error::{Error, DbError};

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

/// Проверяет результат вызова FFI функции и возвращает либо успех в случае, если результат равен `0`,
/// либо [неизвестную ошибку][1] базы данных. Используется в случаях, когда необходимо преобразовать
/// ошибку базы данных в тип Rust, но хендла ошибки еще или уже нет.
///
/// [1]: ../error/enum.DbError.html#variant.Unknown
fn check(native: c_int) -> Result<()> {
  return match native {
    0 => Ok(()),
    e => Err(Error::Db(DbError::Unknown(e as isize)))
  };
}
