//! Содержит типы для работы с большими объектами.

use {Connection, Result};
use convert::FromDB;
use error::Error;
use types::Type;

mod blob;
mod bfile;

pub use self::blob::{Blob, BlobWriter};
pub use self::bfile::{BFile, BFileReader};

trait LobPrivate<'a> : Sized + 'a {
  fn new(raw: &[u8], conn: &'a Connection) -> Self;
}

impl<'conn> FromDB<'conn> for Blob<'conn> {
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    match ty {
      Type::BLOB => Ok(Blob::new(raw, conn)),
      t => Err(Error::Conversion(t)),
    }
  }
}
impl<'conn> FromDB<'conn> for BFile<'conn> {
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    match ty {
      Type::BFILEE => Ok(BFile::new(raw, conn)),
      t => Err(Error::Conversion(t)),
    }
  }
}