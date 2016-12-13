
mod conn;
mod hndl;
mod stmt;

use super::types::Handle;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::stmt::*;

pub trait HandleType {
  const ID: Handle;
}

pub enum OCIEnv {}
pub enum OCIError {}    impl HandleType for OCIError   { const ID: Handle = Handle::Error; }
pub enum OCIServer {}   impl HandleType for OCIServer  { const ID: Handle = Handle::Server; }
pub enum OCISvcCtx {}   impl HandleType for OCISvcCtx  { const ID: Handle = Handle::SvcCtx; }
pub enum OCISession {}  impl HandleType for OCISession { const ID: Handle = Handle::Session; }
pub enum OCIStmt {}     impl HandleType for OCIStmt    { const ID: Handle = Handle::Stmt; }
pub enum OCISnapshot {}