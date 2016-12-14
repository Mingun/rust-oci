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

pub enum OCIEnv {}
pub enum OCIError {}    impl HandleType for OCIError   { const ID: Handle = Handle::Error; }
pub enum OCIServer {}   impl HandleType for OCIServer  { const ID: Handle = Handle::Server; }
pub enum OCISvcCtx {}   impl HandleType for OCISvcCtx  { const ID: Handle = Handle::SvcCtx; }
pub enum OCISession {}  impl HandleType for OCISession { const ID: Handle = Handle::Session; }
pub enum OCIStmt {}     impl HandleType for OCIStmt    { const ID: Handle = Handle::Stmt; }

pub trait DescriptorType {
  const ID: Descriptor;
}
pub enum OCILobLocator {} impl DescriptorType for OCILobLocator { const ID: Descriptor = Descriptor::Lob; }
pub enum OCISnapshot {}   impl DescriptorType for OCISnapshot   { const ID: Descriptor = Descriptor::Snapshot; }