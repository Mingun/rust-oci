//! Содержит типы для работы с большими бинарными объектами.
use std::io;

use {Connection, Result};
use ffi::native::lob::{Lob, LobImpl, LobPiece, LobOpenMode};

use super::LobPrivate;

//-------------------------------------------------------------------------------------------------
/// Указатель на большой бинарный объект (BLOB).
#[derive(Debug)]
pub struct Blob<'conn> {
  /// FFI объект для типобезопасного взаимодействия с базой
  impl_: LobImpl<'conn, Lob>,
}
impl<'conn> Blob<'conn> {
  /// Получает количество байт, содержащихся в данном объекте в данный момент.
  #[inline]
  pub fn len(&self) -> Result<u64> {
    self.impl_.len().map_err(Into::into)
  }
  /// Получает максимальное количество байт, которое может быть сохранено в данном объекте.
  /// В зависимости от настроек сервера базы данных данное значение может варьироваться от
  /// 8 до 128 терабайт (TB).
  #[inline]
  pub fn capacity(&self) -> Result<u64> {
    self.impl_.capacity().map_err(Into::into)
  }
  /// Укорачивает данный объект до указанной длины. В случае, если нова длина больше предыдущей, будет
  /// возвращена ошибка (таким образом. данную функцию нельзя использовать для увеличения размера LOB).
  ///
  /// # Производительность
  /// Необходимо учитывать, что в случае частой записи предпочтительней делать ее через специальный
  /// объект-писатель, который можно получить из данного объекта вызовом функции [`new_writer()`](#function.new_writer).
  /// Если поступить таким образом, то обновление функциональных и доменных индексов базы данных (если они
  /// есть) для данного большого объекта будет отложено до тех пор, пока объект-писатель не будет уничтожен.
  /// При вызове же данной функции обновление данных индексов произойдет сразу же по окончании вызова, что
  /// может сильно снизить производительность.
  #[inline]
  pub fn trim(&mut self, len: u64) -> Result<()> {
    self.impl_.trim(len).map_err(Into::into)
  }

  /// Создает писателя в данный бинарный объект. Преимущество использования писателя вместо прямой записи
  /// в объект в том, что функциональные и доменные индексы базы данных (если они есть) для данного большого
  /// объекта будут обновлены только после уничтожения писателя, а не при каждой записи в объект, что в
  /// лучшую сторону сказывается на производительности.
  #[inline]
  pub fn new_writer(&'conn mut self) -> Result<BlobWriter<'conn>> {
    try!(self.impl_.open(LobOpenMode::WriteOnly));
    Ok(BlobWriter { lob: self })
  }
}
impl<'conn> LobPrivate<'conn> for Blob<'conn> {
  fn new(raw: &[u8], conn: &'conn Connection) -> Self {
    let p = raw.as_ptr() as *const *mut Lob;
    let locator = unsafe { *p as *mut Lob };

    Blob { impl_: LobImpl::from(conn, locator) }
  }
}
impl<'conn> io::Read for Blob<'conn> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // Параметр charset игнорируется для бинарных объектов
    self.impl_.read(0, LobPiece::One, 0, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }
}
impl<'conn> io::Write for Blob<'conn> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    // Параметр charset игнорируется для бинарных объектов
    self.impl_.write(0, LobPiece::One, 0, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
//-------------------------------------------------------------------------------------------------
/// Позволяет писать в большой бинарный объект, не вызывая пересчета индексов после каждой записи.
/// Индексы будут пересчитаны только после уничтожения данного объекта.
pub struct BlobWriter<'lob> {
  lob: &'lob mut Blob<'lob>,
}
impl<'lob> io::Write for BlobWriter<'lob> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.lob.write(buf)
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
impl<'lob> Drop for BlobWriter<'lob> {
  fn drop(&mut self) {
    self.lob.impl_.close().expect("Error when close LOB");
  }
}