//! Модуль, содержащий код для связывания с C интерфейсом OCI.

macro_rules! descriptor {
  ($kind:ty, $name:ident) => (
    #[derive(Debug)]
    pub enum $name {}
    impl $crate::ffi::DescriptorType for $name {
      const ID: $crate::ffi::types::Descriptor = $crate::ffi::types::Descriptor::$name;
    }
    impl $kind for $name {}
  );
}

mod bind;
mod conn;
mod hndl;
mod misc;
mod stmt;
pub mod lob;
pub mod num;
pub mod time;

use ffi::{HandleType, DescriptorType};// признаки хендла/дескриптора
use ffi::{ErrorHandle, VersionHandle, AttrHandle, ParamHandle, InterruptHandle};// Дополнительные свойства хендлов/дескрипторов
use ffi::types::Handle;
use ffi::types::Descriptor;

pub use self::bind::*;
pub use self::conn::*;
pub use self::hndl::*;
pub use self::misc::*;
pub use self::stmt::*;


#[derive(Debug)] pub enum OCIDescribe {} impl HandleType for OCIDescribe { const ID: Handle = Handle::Describe; }
#[derive(Debug)] pub enum OCIEnv {}
impl ErrorHandle for OCIEnv { const ID: Handle = Handle::Env; }
#[derive(Debug)] pub enum OCIError {}    impl HandleType for OCIError    { const ID: Handle = Handle::Error; }
impl ErrorHandle for OCIError { const ID: Handle = Handle::Error; }
#[derive(Debug)] pub enum OCIServer {}   impl HandleType for OCIServer   { const ID: Handle = Handle::Server; }
impl VersionHandle for OCIServer {}
impl InterruptHandle for OCIServer {}
#[derive(Debug)] pub enum OCISvcCtx {}   impl HandleType for OCISvcCtx   { const ID: Handle = Handle::SvcCtx; }
impl VersionHandle for OCISvcCtx {}
impl InterruptHandle for OCISvcCtx {}
#[derive(Debug)] pub enum OCISession {}  impl HandleType for OCISession  { const ID: Handle = Handle::Session; }
#[derive(Debug)] pub enum OCIStmt {}
impl AttrHandle  for OCIStmt { const ID: Handle = Handle::Stmt; }
impl ParamHandle for OCIStmt { const ID: Handle = Handle::Stmt; }
#[derive(Debug)] pub enum OCIBind {}     impl HandleType for OCIBind     { const ID: Handle = Handle::Bind; }
#[derive(Debug)] pub enum OCIDefine {}   impl HandleType for OCIDefine   { const ID: Handle = Handle::Define; }


#[derive(Debug)] pub enum OCISnapshot {}          impl DescriptorType for OCISnapshot           { const ID: Descriptor = Descriptor::Snapshot; }
//#[derive(Debug)] pub enum OCIResult {}            impl DescriptorType for OCIResult             { const ID: Descriptor = Descriptor::; }
//#[derive(Debug)] pub enum OCILobRegion {}         impl DescriptorType for OCILobRegion          { const ID: Descriptor = Descriptor::; }
#[derive(Debug)] pub enum OCIParam {}             impl DescriptorType for OCIParam              { const ID: Descriptor = Descriptor::Param; }
#[derive(Debug)] pub enum OCIComplexObjectComp {} impl DescriptorType for OCIComplexObjectComp  { const ID: Descriptor = Descriptor::ComplexObjectComp; }
#[derive(Debug)] pub enum OCIRowid {}             impl DescriptorType for OCIRowid              { const ID: Descriptor = Descriptor::RowID; }
#[derive(Debug)] pub enum OCIUcb {}               impl DescriptorType for OCIUcb                { const ID: Descriptor = Descriptor::UCB; }
#[derive(Debug)] pub enum OCIServerDNs {}         impl DescriptorType for OCIServerDNs          { const ID: Descriptor = Descriptor::ServerDN; }

#[derive(Debug)] pub enum OCIType {}
