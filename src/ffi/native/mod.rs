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

use super::types::Handle;
use super::types::Descriptor;
pub use self::bind::*;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::misc::*;
pub use self::stmt::*;
pub use self::lob::*;
pub use self::num::*;

/// Тип, реализующий данный типаж, может быть передан в функцию `OCIHandleAlloc` для создания хендла.
/// Ассоциированная константа `ID` указывает тип хендла, который будет передан в функцию.
pub trait HandleType {
  const ID: Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функцию `OCIErrorGet` для получения информации об ошибке
pub trait ErrorHandle {
  const ID: Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функции `OCIAttrGet/OCIAttrSet` для получения или установки атрибута
pub trait AttrHandle {
  const ID: Handle;
}
/// Тип, реализующий данный типаж, может быть передан в функцию `OCIParamGet` для получения информации о параметре
pub trait ParamHandle {
  const ID: Handle;
}

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

/// Тип, реализующий данный типаж, может быть передан в функцию `OCIDescriptorAlloc` для создания дескриптора.
/// Ассоциированная константа `ID` указывает тип дескриптора, который будет передан в функцию.
pub trait DescriptorType {
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