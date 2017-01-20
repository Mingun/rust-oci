//! Содержит типы для работы с большими бинарными объектами.
use std::io;

use {Connection, Result, DbResult};
use types::Charset;
use ffi::native::lob::{Lob, LobImpl, LobPiece, LobOpenMode, CharsetForm};

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

  /// Создает читателя данного бинарного объекта. Каждый вызов метода `read` читателя читает очередную порцию данных.
  #[inline]
  pub fn new_reader<'lob>(&'lob mut self) -> Result<BlobReader<'lob, 'conn>> {
    try!(self.impl_.open(LobOpenMode::ReadOnly));
    Ok(BlobReader { lob: self, piece: LobPiece::First })
  }
  /// Создает писателя в данный бинарный объект. Преимущество использования писателя вместо прямой записи
  /// в объект в том, что функциональные и доменные индексы базы данных (если они есть) для данного большого
  /// объекта будут обновлены только после уничтожения писателя, а не при каждой записи в объект, что в
  /// лучшую сторону сказывается на производительности.
  ///
  /// В пределах одной транзакции один BLOB может быть открыт только единожды, независимо от того, сколько
  /// локаторов (которые представляет данный класс) на него существует.
  #[inline]
  pub fn new_writer<'lob>(&'lob mut self) -> Result<BlobWriter<'lob, 'conn>> {
    try!(self.impl_.open(LobOpenMode::WriteOnly));
    Ok(BlobWriter { lob: self, piece: LobPiece::First })
  }
  fn close(&mut self, piece: LobPiece) -> DbResult<()> {
    // Если LOB был прочитан/записан не полностью, то отменяем запросы на чтение/запись и восстанавливаемся
    if piece != LobPiece::Last {
      try!(self.impl_.break_());
      try!(self.impl_.reset());
    }
    self.impl_.close()
  }
}
impl<'conn> LobPrivate<'conn> for Blob<'conn> {
  fn new(raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    let p = raw.as_ptr() as *const *mut Lob;
    let locator = unsafe { *p as *mut Lob };

    Ok(Blob { impl_: LobImpl::from(conn, locator) })
  }
}
impl<'conn> io::Read for Blob<'conn> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // Количество того, сколько читать и сколько было реально прочитано.
    let mut readed = buf.len() as u64;
    // Параметры charset и form игнорируется для бинарных объектов
    match self.impl_.read_impl(0, LobPiece::One, Charset::Default, CharsetForm::Implicit, buf, &mut readed) {
      // Не может быть прочитано больше, чем было запрошено, а то, что было запрошено,
      // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
      Ok(_) => Ok(readed as usize),
      Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
  }
}
impl<'conn> io::Write for Blob<'conn> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    // Количество того, сколько писать и сколько было реально записано.
    let mut writed = buf.len() as u64;
    // Параметры charset и form игнорируется для бинарных объектов
    match self.impl_.write_impl(0, LobPiece::One, Charset::Default, CharsetForm::Implicit, buf, &mut writed) {
      // Не может быть записано больше, чем было запрошено, а то, что было запрошено,
      // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
      Ok(_) => Ok(writed as usize),
      Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
//-------------------------------------------------------------------------------------------------
/// Позволяет писать в большой бинарный объект, не вызывая пересчета индексов после каждой записи.
/// Индексы будут пересчитаны только после уничтожения данного объекта.
#[derive(Debug)]
pub struct BlobWriter<'lob, 'conn: 'lob> {
  lob: &'lob mut Blob<'conn>,
  piece: LobPiece,
}
impl<'lob, 'conn: 'lob> BlobWriter<'lob, 'conn> {
  /// Получает `BLOB`, записываемый данным писателем.
  pub fn lob(&mut self) -> &mut Blob<'conn> {
    self.lob
  }
}
impl<'lob, 'conn: 'lob> BlobWriter<'lob, 'conn> {
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
impl<'lob, 'conn: 'lob> io::Write for BlobWriter<'lob, 'conn> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    // Параметры charset и form игнорируется для бинарных объектов
    let (res, piece) = self.lob.impl_.write(self.piece, Charset::Default, CharsetForm::Implicit, buf);
    self.piece = piece;
    res
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
impl<'lob, 'conn: 'lob> Drop for BlobWriter<'lob, 'conn> {
  fn drop(&mut self) {
    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.lob.close(self.piece);//.expect("Error when close BLOB writer");
  }
}

//-------------------------------------------------------------------------------------------------
/// Позволяет читать из большой бинарного объекта в потоковом режиме. Каждый вызов `read` читает очередную порцию данных.
#[derive(Debug)]
pub struct BlobReader<'lob, 'conn: 'lob> {
  lob: &'lob mut Blob<'conn>,
  piece: LobPiece,
}
impl<'lob, 'conn: 'lob> BlobReader<'lob, 'conn> {
  /// Получает `BLOB`, читаемый данным читателем.
  pub fn lob(&mut self) -> &mut Blob<'conn> {
    self.lob
  }
}
impl<'lob, 'conn: 'lob> io::Read for BlobReader<'lob, 'conn> {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // Параметры charset и form игнорируется для бинарных объектов
    let (res, piece) = self.lob.impl_.read(self.piece, Charset::Default, CharsetForm::Implicit, buf);
    self.piece = piece;
    res
  }
}
impl<'lob, 'conn: 'lob> Drop for BlobReader<'lob, 'conn> {
  fn drop(&mut self) {
    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.lob.close(self.piece);//.expect("Error when close BLOB reader");
  }
}