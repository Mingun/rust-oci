//! Содержит типы для работы с большими объектами.

use {Connection, Result};
use convert::FromDB;
use error::Error;
use types::Type;

mod blob;

pub use self::blob::{Blob, BlobWriter};

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