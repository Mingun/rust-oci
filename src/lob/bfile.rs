//! Содержит типы для работы с файловыми бинарными объектами.
use std::io;

use {Connection, Result};
use error::DbError::NeedData;
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
pub struct BFileReader<'lob> {
  lob: &'lob mut BFile<'lob>,
  piece: LobPiece,
}
impl<'lob> io::Read for BFileReader<'lob> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // Если в прошлый раз при чтении был достигнут конец потока, возвращаем 0
    if self.piece == LobPiece::Last {
      return Ok(0);
    }
    // Количество того, сколько читать и сколько было реально прочитано.
    // В случае потокового чтения можно передать 0, это значит, что мы собираемся прочитать LOB до конца.
    let mut readed = 0;
    // Параметр charset игнорируется для бинарных объектов
    let res = self.lob.impl_.read(0, self.piece, Charset::Default, buf, &mut readed);
    self.piece = LobPiece::Next;

    match res {
      Ok(_) => {
        // Чтение закончено, теперь будем постоянно возвращать 0
        self.piece = LobPiece::Last;
        Ok(readed as usize)
      },
      // Не может быть прочитано больше, чем было запрошено, а то, что было запрошено,
      // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
      Err(NeedData) => Ok(readed as usize),
      Err(e) => Err(io::Error::new(io::ErrorKind::Other, e))
    }
  }
}
impl<'lob> Drop for BFileReader<'lob> {
  fn drop(&mut self) {
    self.lob.impl_.close().expect("Error when close LOB");
  }
}