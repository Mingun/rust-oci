//! Содержит типы для работы с большими бинарными объектами.
use std::io;

use {Connection, Result};
use ffi::native::lob::{Lob, LobImpl, LobPiece, LobOpenMode};

use super::{Bytes, LobPrivate};

//-------------------------------------------------------------------------------------------------
/// Указатель на большой бинарный объект (BLOB).
#[derive(Debug, PartialEq, Eq)]
pub struct Blob<'conn> {
  /// FFI объект для типобезопасного взаимодействия с базой
  impl_: LobImpl<'conn, Lob>,
}
impl<'conn> Blob<'conn> {
  /// Получает количество байт, содержащихся в данном объекте в данный момент.
  #[inline]
  pub fn len(&self) -> Result<Bytes> {
    let len = try!(self.impl_.len());
    Ok(Bytes(len))
  }
  /// Получает максимальное количество байт, которое может быть сохранено в данном объекте.
  /// В зависимости от настроек сервера базы данных данное значение может варьироваться от
  /// 8 до 128 терабайт (TB).
  #[inline]
  pub fn capacity(&self) -> Result<Bytes> {
    let len = try!(self.impl_.capacity());
    Ok(Bytes(len))
  }
  /// For LOBs with storage parameter `BASICFILE`, the amount of a chunk's space that is used to store
  /// the internal LOB value. This is the amount that users should use when reading or writing the LOB
  /// value. If possible, users should start their writes at chunk boundaries, such as the beginning of
  /// a chunk, and write a chunk at a time.
  ///
  /// For LOBs with storage parameter `SECUREFILE`, chunk size is an advisory size and is provided for
  /// backward compatibility.
  ///
  /// When creating a table that contains an internal LOB, the user can specify the chunking factor,
  /// which can be a multiple of Oracle Database blocks. This corresponds to the chunk size used by
  /// the LOB data layer when accessing and modifying the LOB value. Part of the chunk is used to store
  /// system-related information, and the rest stores the LOB value. This function returns the amount
  /// of space used in the LOB chunk to store the LOB value. Performance is improved if the application
  /// issues read or write requests using a multiple of this chunk size. For writes, there is an added
  /// benefit because LOB chunks are versioned and, if all writes are done on a chunk basis, no extra
  /// versioning is done or duplicated. Users could batch up the write until they have enough for a chunk
  /// instead of issuing several write calls for the same chunk.
  #[inline]
  pub fn get_chunk_size(&self) -> Result<Bytes> {
    let size = try!(self.impl_.get_chunk_size());
    Ok(Bytes(size as u64))
  }
  /// Укорачивает данный объект до указанной длины. В случае, если новая длина больше предыдущей, будет
  /// возвращена ошибка (таким образом, данную функцию нельзя использовать для увеличения размера LOB).
  ///
  /// # Производительность
  /// Необходимо учитывать, что в случае частой записи предпочтительней делать ее через специальный
  /// объект-писатель, который можно получить из данного объекта вызовом функции [`new_writer()`](#function.new_writer).
  /// Если поступить таким образом, то обновление функциональных и доменных индексов базы данных (если они
  /// есть) для данного большого объекта будет отложено до тех пор, пока объект-писатель не будет уничтожен.
  /// При вызове же данной функции обновление данных индексов произойдет сразу же по окончании вызова, что
  /// может сильно снизить производительность.
  #[inline]
  pub fn trim(&mut self, len: Bytes) -> Result<()> {
    self.impl_.trim(len.0).map_err(Into::into)
  }
  /// Заполняет LOB, начиная с указанного индекса, указанным количеством нулей. После завершения
  /// работы в `count` будет записано реальное количество  очищенных байт.
  ///
  /// # Производительность
  /// Необходимо учитывать, что в случае частой записи предпочтительней делать ее через специальный
  /// объект-писатель, который можно получить из данного объекта вызовом функции [`new_writer()`](#function.new_writer).
  /// Если поступить таким образом, то обновление функциональных и доменных индексов базы данных (если они
  /// есть) для данного большого объекта будет отложено до тех пор, пока объект-писатель не будет уничтожен.
  /// При вызове же данной функции обновление данных индексов произойдет сразу же по окончании вызова, что
  /// может сильно снизить производительность.
  #[inline]
  pub fn erase(&mut self, offset: Bytes, count: &mut Bytes) -> Result<()> {
    self.impl_.erase(offset.0, &mut count.0).map_err(Into::into)
  }

  /// Создает писателя в данный бинарный объект. Преимущество использования писателя вместо прямой записи
  /// в объект в том, что функциональные и доменные индексы базы данных (если они есть) для данного большого
  /// объекта будут обновлены только после уничтожения писателя, а не при каждой записи в объект, что в
  /// лучшую сторону сказывается на производительности.
  ///
  /// В пределах одной транзакции один BLOB может быть открыт только единожды, независимо от того, сколько
  /// локаторов (которые представляет данный класс) на него существует.
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
impl<'lob> BlobWriter<'lob> {
  /// Укорачивает данный объект до указанной длины. В случае, если новая длина больше предыдущей, будет
  /// возвращена ошибка (таким образом, данную функцию нельзя использовать для увеличения размера LOB).
  ///
  /// # Производительность
  /// Необходимо учитывать, что в случае частой записи предпочтительней делать ее через специальный
  /// объект-писатель, который можно получить из данного объекта вызовом функции [`new_writer()`](#function.new_writer).
  /// Если поступить таким образом, то обновление функциональных и доменных индексов базы данных (если они
  /// есть) для данного большого объекта будет отложено до тех пор, пока объект-писатель не будет уничтожен.
  /// При вызове же данной функции обновление данных индексов произойдет сразу же по окончании вызова, что
  /// может сильно снизить производительность.
  #[inline]
  pub fn trim(&mut self, len: Bytes) -> Result<()> {
    self.lob.trim(len)
  }
  /// Заполняет LOB, начиная с указанного индекса, указанным количеством нулей. После завершения
  /// работы в `count` будет записано реальное количество очищенных байт.
  #[inline]
  pub fn erase(&mut self, offset: Bytes, count: &mut Bytes) -> Result<()> {
    self.lob.erase(offset, count)
  }
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