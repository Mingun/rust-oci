//! Содержит типы для работы с файловыми бинарными объектами.
use std::io;

use {Connection, Result, DbResult};
use types::Charset;
use ffi::native::lob::{File, LobImpl, LobPiece, LobOpenMode};

use super::{Bytes, LobPrivate};

//-------------------------------------------------------------------------------------------------
/// Указатель на большой бинарный объект, представленный внешним по отношению к базе данных файлом
/// (BFILE). Данный объект доступен только для чтения.
#[derive(Debug, PartialEq, Eq)]
pub struct BFile<'conn> {
  /// FFI объект для типобезопасного взаимодействия с базой
  impl_: LobImpl<'conn, File>,
}
impl<'conn> BFile<'conn> {
  /// Получает количество байт, содержащихся в данном объекте в данный момент.
  #[inline]
  pub fn len(&self) -> Result<Bytes> {
    let len = try!(self.impl_.len());
    Ok(Bytes(len))
  }
  /// Проверяет, что указанный файл с данными существует на файловой системе сервера базы данных.
  #[inline]
  pub fn is_exist(&self) -> Result<bool> {
    self.impl_.is_exist().map_err(Into::into)
  }
  /// Создает читателя данного файлового бинарного объекта. В отличие от BLOB-ов, файловые объект должны
  /// быть явно открыты, чтобы выполнять из них чтение.
  #[inline]
  pub fn new_reader(&'conn mut self) -> Result<BFileReader<'conn>> {
    try!(self.impl_.open(LobOpenMode::ReadOnly));
    Ok(BFileReader { lob: self, piece: LobPiece::First })
  }
  fn close(&mut self, piece: LobPiece) -> DbResult<()> {
    // Если LOB был прочитан не полностью, то отменяем запросы на чтение и восстанавливаемся
    if piece != LobPiece::Last {
      try!(self.impl_.break_());
      try!(self.impl_.reset());
    }
    self.impl_.close()
  }
}
impl<'conn> LobPrivate<'conn> for BFile<'conn> {
  fn new(raw: &[u8], conn: &'conn Connection) -> Self {
    let p = raw.as_ptr() as *const *mut File;
    let locator = unsafe { *p as *mut File };

    BFile { impl_: LobImpl::from(conn, locator) }
  }
}

//-------------------------------------------------------------------------------------------------
/// Позволяет читать из файлового объекта. При уничтожении закрывает файловый объект.
#[derive(Debug)]
pub struct BFileReader<'lob> {
  lob: &'lob mut BFile<'lob>,
  piece: LobPiece,
}
impl<'lob> io::Read for BFileReader<'lob> {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // Параметр charset игнорируется для бинарных объектов
    let (res, piece) = self.lob.impl_.read(self.piece, Charset::Default, buf);
    self.piece = piece;
    res
  }
}
impl<'lob> Drop for BFileReader<'lob> {
  fn drop(&mut self) {
    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.lob.close(self.piece);//.expect("Error when close BFILE reader");
  }
}