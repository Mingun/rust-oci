
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

fn check(native: c_int) -> Result<()> {
  return match native {
    0 => Ok(()),
    e => Err(Error::Db(DbError::Unknown(e as isize)))
  };
}
