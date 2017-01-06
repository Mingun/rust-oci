//! Модуль, содержащий код для связывания с C интерфейсом OCI.

macro_rules! descriptor {
  ($kind:ident, $name:ident) => (
    #[derive(Debug)]
    pub enum $name {}
    impl DescriptorType for $name { const ID: Descriptor = Descriptor::$name; }
    impl $kind for $name {}
  );
}

mod bind;
mod conn;
mod hndl;
mod misc;
mod stmt;
mod lob;
mod num;
pub mod time;

use ffi::types::Handle;
use ffi::types::Descriptor;

pub use self::bind::*;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::misc::*;
pub use self::stmt::*;
pub use self::lob::*;
pub use self::num::*;

/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIHandleAlloc`][new] для создания хендла.
/// Ассоциированная константа `ID` указывает тип хендла, который будет передан в функцию.
///
/// [new]: https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#GUID-C5BF55F7-A110-4CB5-9663-5056590F12B5
pub trait HandleType {
  /// Вид хендла, используемый типом, реализующим данный типаж.
  const ID: Handle;
}

/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIErrorGet`][1] для получения информации об ошибке.
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17287
pub trait ErrorHandle {
  /// Тип хендла, из которого получается описание ошибки. Последний параметр функции [`OCIErrorGet`][1] (`type`).
  ///
  /// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17287
  const ID: Handle;
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
  const ID: Handle;
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
  const ID: Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функции [`OCIServerRelease`][1]/[`OCIServerVersion`][2]
/// для получения информации о версии сервера Oracle. Ассоциированная константа [`ID`][id] передается
/// предпоследним/последним параметром в эти функции (`hndltype`).
///
/// [1]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17293
/// [2]: http://docs.oracle.com/database/122/LNOCI/miscellaneous-functions.htm#LNOCI17294
/// [id]: ./trait.HandleType.html#associatedconstant.ID
pub trait VersionHandle : HandleType {}

#[derive(Debug)] pub enum OCIDescribe {} impl HandleType for OCIDescribe { const ID: Handle = Handle::Describe; }
#[derive(Debug)] pub enum OCIEnv {}
impl ErrorHandle for OCIEnv { const ID: Handle = Handle::Env; }
#[derive(Debug)] pub enum OCIError {}    impl HandleType for OCIError    { const ID: Handle = Handle::Error; }
impl ErrorHandle for OCIError { const ID: Handle = Handle::Error; }
#[derive(Debug)] pub enum OCIServer {}   impl HandleType for OCIServer   { const ID: Handle = Handle::Server; }
impl VersionHandle for OCIServer {}
#[derive(Debug)] pub enum OCISvcCtx {}   impl HandleType for OCISvcCtx   { const ID: Handle = Handle::SvcCtx; }
impl VersionHandle for OCISvcCtx {}
#[derive(Debug)] pub enum OCISession {}  impl HandleType for OCISession  { const ID: Handle = Handle::Session; }
#[derive(Debug)] pub enum OCIStmt {}
impl AttrHandle  for OCIStmt { const ID: Handle = Handle::Stmt; }
impl ParamHandle for OCIStmt { const ID: Handle = Handle::Stmt; }
#[derive(Debug)] pub enum OCIBind {}     impl HandleType for OCIBind     { const ID: Handle = Handle::Bind; }
#[derive(Debug)] pub enum OCIDefine {}   impl HandleType for OCIDefine   { const ID: Handle = Handle::Define; }

/// Тип, реализующий данный типаж, может быть передан в функцию [`OCIDescriptorAlloc`][new] для создания дескриптора.
/// Ассоциированная константа `ID` указывает тип дескриптора, который будет передан в функцию.
///
/// [new]: http://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-functions.htm#LNOCI17132
pub trait DescriptorType {
  /// Вид дескриптора, используемый типом, реализующим данный типаж.
  const ID: Descriptor;
}

#[derive(Debug)] pub enum OCISnapshot {}          impl DescriptorType for OCISnapshot           { const ID: Descriptor = Descriptor::Snapshot; }
//#[derive(Debug)] pub enum OCIResult {}            impl DescriptorType for OCIResult             { const ID: Descriptor = Descriptor::; }
#[derive(Debug)] pub enum OCILobLocator {}        impl DescriptorType for OCILobLocator         { const ID: Descriptor = Descriptor::Lob; }//FIXME: Может также быть и File
//#[derive(Debug)] pub enum OCILobRegion {}         impl DescriptorType for OCILobRegion          { const ID: Descriptor = Descriptor::; }
#[derive(Debug)] pub enum OCIParam {}             impl DescriptorType for OCIParam              { const ID: Descriptor = Descriptor::Param; }
#[derive(Debug)] pub enum OCIComplexObjectComp {} impl DescriptorType for OCIComplexObjectComp  { const ID: Descriptor = Descriptor::ComplexObjectComp; }
#[derive(Debug)] pub enum OCIRowid {}             impl DescriptorType for OCIRowid              { const ID: Descriptor = Descriptor::RowID; }
#[derive(Debug)] pub enum OCIUcb {}               impl DescriptorType for OCIUcb                { const ID: Descriptor = Descriptor::UCB; }
#[derive(Debug)] pub enum OCIServerDNs {}         impl DescriptorType for OCIServerDNs          { const ID: Descriptor = Descriptor::ServerDN; }

#[derive(Debug)] pub enum OCIType {}
/*
pub trait OCILobLocator : DescriptorType {}
descriptor!(OCILobLocator, Lob);
descriptor!(OCILobLocator, File);
*/
/*
#[derive(Debug)]
#[repr(C)]
pub enum OCILobLocator {
  Lob {},
  File {},
}*/