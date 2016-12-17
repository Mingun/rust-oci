//! Модуль, содержащий код для связывания с C интерфейсов OCI.

mod conn;
mod hndl;
mod stmt;
mod lob;

use super::types::Handle;
use super::types::Descriptor;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::stmt::*;
pub use self::lob::*;

pub trait HandleType {
  const ID: Handle;
}
/// Теп, реализующий данный типаж, может быть передан в функцию `OCIErrorGet` для получения информации об ошибке
pub trait ErrorHandle {
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

pub trait DescriptorType {
  const ID: Descriptor;
}
#[derive(Debug)] pub enum OCILobLocator {} impl DescriptorType for OCILobLocator { const ID: Descriptor = Descriptor::Lob; }
#[derive(Debug)] pub enum OCISnapshot {}   impl DescriptorType for OCISnapshot   { const ID: Descriptor = Descriptor::Snapshot; }