//! Модуль, содержащий код для связывания с C интерфейсов OCI.

mod bind;
mod conn;
mod hndl;
mod stmt;
mod lob;

use super::types::Handle;
use super::types::Descriptor;
pub use self::bind::*;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::stmt::*;
pub use self::lob::*;

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

#[derive(Debug)] pub enum OCIEnv {}
impl ErrorHandle for OCIEnv { const ID: Handle = Handle::Env; }
#[derive(Debug)] pub enum OCIError {}    impl HandleType for OCIError   { const ID: Handle = Handle::Error; }
impl ErrorHandle for OCIError { const ID: Handle = Handle::Error; }
#[derive(Debug)] pub enum OCIServer {}   impl HandleType for OCIServer  { const ID: Handle = Handle::Server; }
#[derive(Debug)] pub enum OCISvcCtx {}   impl HandleType for OCISvcCtx  { const ID: Handle = Handle::SvcCtx; }
#[derive(Debug)] pub enum OCISession {}  impl HandleType for OCISession { const ID: Handle = Handle::Session; }
#[derive(Debug)] pub enum OCIStmt {}
impl AttrHandle  for OCIStmt { const ID: Handle = Handle::Stmt; }
impl ParamHandle for OCIStmt { const ID: Handle = Handle::Stmt; }

/// Тип, реализующий данный типаж, может быть передан в функцию `OCIDescriptorAlloc` для создания дескриптора.
/// Ассоциированная константа `ID` указывает тип дескриптора, который будет передан в функцию.
pub trait DescriptorType {
  const ID: Descriptor;
}
#[derive(Debug)] pub enum OCILobLocator {} impl DescriptorType for OCILobLocator { const ID: Descriptor = Descriptor::Lob; }
#[derive(Debug)] pub enum OCISnapshot {}   impl DescriptorType for OCISnapshot   { const ID: Descriptor = Descriptor::Snapshot; }
#[derive(Debug)] pub enum OCIParam {}      impl DescriptorType for OCIParam      { const ID: Descriptor = Descriptor::Param; }
#[derive(Debug)] pub enum OCIBind {}
#[derive(Debug)] pub enum OCIDefine {}