//! Содержит типы для работы с большими объектами.

use {Connection, Result};
use convert::FromDB;
use error::Error;
use types::Type;

mod blob;
mod clob;
mod bfile;

pub use self::blob::{Blob, BlobReader, BlobWriter};
pub use self::clob::{Clob, ClobReader, ClobWriter};
pub use self::bfile::{BFile, BFileReader};

/// Тип, представляющий размер в байтах.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Bytes(pub u64);
/// Тип, представляющий размер в символах. Следует учитывать, что "символ" в понимании Oracle -- это
/// один юнит кодировки UTF-16, занимающий 2 байта. Таким образом, кодовые точки Юникода, представленные
/// [суррогатными парами][utf-16] в UTF-16, считаются, как 2 символа.
///
/// [utf-16]: https://ru.wikipedia.org/wiki/UTF-16#.D0.9F.D1.80.D0.B8.D0.BD.D1.86.D0.B8.D0.BF_.D0.BA.D0.BE.D0.B4.D0.B8.D1.80.D0.BE.D0.B2.D0.B0.D0.BD.D0.B8.D1.8F
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Chars(pub u64);

trait LobPrivate<'a> : Sized + 'a {
  fn new(raw: &[u8], conn: &'a Connection) -> Result<Self>;
}

impl<'conn> FromDB<'conn> for Blob<'conn> {
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    match ty {
      Type::BLOB => Blob::new(raw, conn),
      t => Err(Error::Conversion(t)),
    }
  }
}
impl<'conn> FromDB<'conn> for Clob<'conn> {
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    match ty {
      Type::CLOB => Clob::new(raw, conn),
      t => Err(Error::Conversion(t)),
    }
  }
}
impl<'conn> FromDB<'conn> for BFile<'conn> {
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    match ty {
      Type::BFILEE => BFile::new(raw, conn),
      t => Err(Error::Conversion(t)),
    }
  }
}