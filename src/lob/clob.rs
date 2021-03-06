//! Содержит типы для работы с большими символьными объектами.
use std::io;

use {Connection, Result, DbResult};
use types::Charset;
use ffi::native::lob::{Lob, LobImpl, LobOpenMode, CharsetForm};
use ffi::types::Piece;

use super::{Bytes, Chars, LobPrivate};

//-------------------------------------------------------------------------------------------------
/// Указатель на большой символьный объект (CLOB или NCLOB).
#[derive(Debug, PartialEq, Eq)]
pub struct Clob<'conn> {
  /// FFI объект для типобезопасного взаимодействия с базой
  impl_: LobImpl<'conn, Lob>,
  /// Вид символьного объекта: в кодировке базы данных (CLOB) или в национальной кодировке (NCLOB).
  form: CharsetForm,
}
impl<'conn> Clob<'conn> {
  /// Получает количество символов, содержащихся в данном объекте в данный момент.
  ///
  /// Следует учитывать, что "символ" в понимании Oracle -- это один юнит кодировки UTF-16, занимающий
  /// 2 байта. Таким образом, кодовые точки Юникода, представленные [суррогатными парами][utf-16] в UTF-16,
  /// считаются, как 2 символа.
  ///
  /// [utf-16]: https://ru.wikipedia.org/wiki/UTF-16#.D0.9F.D1.80.D0.B8.D0.BD.D1.86.D0.B8.D0.BF_.D0.BA.D0.BE.D0.B4.D0.B8.D1.80.D0.BE.D0.B2.D0.B0.D0.BD.D0.B8.D1.8F
  #[inline]
  pub fn len(&self) -> Result<Chars> {
    let len = try!(self.impl_.len());
    Ok(Chars(len))
  }
  /// Получает максимальное количество *байт*, которое может быть сохранено в данном объекте.
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
  pub fn trim(&mut self, len: Chars) -> Result<()> {
    self.impl_.trim(len.0).map_err(Into::into)
  }
  /// Заполняет LOB, начиная с указанного индекса, указанным количеством пробелов. После завершения
  /// работы в `count` будет записано реальное количество очищенных символов.
  ///
  /// # Производительность
  /// Необходимо учитывать, что в случае частой записи предпочтительней делать ее через специальный
  /// объект-писатель, который можно получить из данного объекта вызовом функции [`new_writer()`](#function.new_writer).
  /// Если поступить таким образом, то обновление функциональных и доменных индексов базы данных (если они
  /// есть) для данного большого объекта будет отложено до тех пор, пока объект-писатель не будет уничтожен.
  /// При вызове же данной функции обновление данных индексов произойдет сразу же по окончании вызова, что
  /// может сильно снизить производительность.
  #[inline]
  pub fn erase(&mut self, offset: Chars, count: &mut Chars) -> Result<()> {
    self.impl_.erase(offset.0, &mut count.0).map_err(Into::into)
  }

  /// Создает читателя данного символьного объекта. Каждый вызов метода `read` читателя читает очередную порцию данных.
  /// Данные читаются из CLOB-а в кодировке `UTF-8`.
  #[inline]
  pub fn new_reader<'lob>(&'lob mut self) -> Result<ClobReader<'lob, 'conn>> {
    self.new_reader_with_charset(Charset::AL32UTF8)
  }
  /// Создает читателя данного символьного объекта. Каждый вызов метода `read` читателя читает очередную порцию данных.
  /// Данные читаются из CLOB-а в указанной кодировке.
  ///
  /// Каждый вызов `read` будет заполнять массив байтами в запрошенной кодировке. Так как стандартные методы Rust для
  /// работы читателем байт как читателем текста предполагают, что представлен в UTF-8, то их нельзя использовать для
  /// данного читателя, т.к. тест будет извлекаться с указанной кодировке.
  #[inline]
  pub fn new_reader_with_charset<'lob>(&'lob mut self, charset: Charset) -> Result<ClobReader<'lob, 'conn>> {
    try!(self.impl_.open(LobOpenMode::ReadOnly));
    Ok(ClobReader { lob: self, piece: Piece::First, charset: charset })
  }
  /// Создает писателя в данный символьный объект. Преимущество использования писателя вместо прямой записи
  /// в объект в том, что функциональные и доменные индексы базы данных (если они есть) для данного большого
  /// объекта будут обновлены только после уничтожения писателя, а не при каждой записи в объект, что в
  /// лучшую сторону сказывается на производительности.
  ///
  /// В пределах одной транзакции один CLOB может быть открыт только единожды, независимо от того, сколько
  /// локаторов (которые представляет данный класс) на него существует.
  #[inline]
  pub fn new_writer<'lob>(&'lob mut self) -> Result<ClobWriter<'lob, 'conn>> {
    self.new_writer_with_charset(Charset::AL32UTF8)
  }
  /// Создает писателя в данный символьный объект, записывающий текстовые данные, представленные в указанной кодировке.
  ///
  /// Преимущество использования писателя вместо прямой записи в объект в том, что функциональные и доменные индексы
  /// базы данных (если они есть) для данного большого объекта будут обновлены только после уничтожения писателя, а не
  /// при каждой записи в объект, что в лучшую сторону сказывается на производительности.
  ///
  /// В пределах одной транзакции один CLOB может быть открыт только единожды, независимо от того, сколько
  /// локаторов (которые представляет данный класс) на него существует.
  #[inline]
  pub fn new_writer_with_charset<'lob>(&'lob mut self, charset: Charset) -> Result<ClobWriter<'lob, 'conn>> {
    try!(self.impl_.open(LobOpenMode::WriteOnly));
    Ok(ClobWriter { lob: self, piece: Piece::First, charset: charset })
  }
  /// Получает кодировку базы данных для данного большого символьного объекта.
  #[inline]
  pub fn charset(&self) -> Result<Charset> {
    self.impl_.charset().map_err(Into::into)
  }
  /// Если CLOB прочитан или записан не полностью, то сообщает базе данных, что дальнейшее чтение/запись не требуются
  /// и закрывает CLOB.
  fn close(&mut self, piece: Piece) -> DbResult<()> {
    // Если LOB был прочитан/записан не полностью, то отменяем запросы на чтение/запись и восстанавливаемся
    if piece != Piece::Last {
      try!(self.impl_.break_());
      try!(self.impl_.reset());
    }
    self.impl_.close()
  }
}
impl<'conn> LobPrivate<'conn> for Clob<'conn> {
  fn new(raw: &[u8], conn: &'conn Connection) -> Result<Self> {
    let p = raw.as_ptr() as *const *mut Lob;
    let locator = unsafe { *p as *mut Lob };
    let impl_ = LobImpl::from(conn, locator);
    let form = try!(impl_.form());

    Ok(Clob { impl_: impl_, form: form })
  }
}
impl<'conn> io::Read for Clob<'conn> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.impl_.read(Piece::One, Charset::AL32UTF8, self.form, buf).0
  }
}
impl<'conn> io::Write for Clob<'conn> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.impl_.write(Piece::One, Charset::AL32UTF8, self.form, buf).0
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
//-------------------------------------------------------------------------------------------------
/// Позволяет писать в большой символьный объект, не вызывая пересчета индексов после каждой записи.
/// Индексы будут пересчитаны только после уничтожения данного объекта.
#[derive(Debug)]
pub struct ClobWriter<'lob, 'conn: 'lob> {
  lob: &'lob mut Clob<'conn>,
  piece: Piece,
  charset: Charset,
}
impl<'lob, 'conn: 'lob> ClobWriter<'lob, 'conn> {
  /// Получает `CLOB`, записываемый данным писателем.
  pub fn lob(&mut self) -> &mut Clob<'conn> {
    self.lob
  }
  /// Укорачивает данный объект до указанной длины. В случае, если новая длина больше предыдущей, будет
  /// возвращена ошибка (таким образом, данную функцию нельзя использовать для увеличения размера LOB).
  #[inline]
  pub fn trim(&mut self, len: Chars) -> Result<()> {
    self.lob.trim(len)
  }
  /// Заполняет LOB, начиная с указанного индекса, указанным количеством нулей. После завершения
  /// работы в `count` будет записано реальное количество  очищенных байт.
  #[inline]
  pub fn erase(&mut self, offset: Chars, count: &mut Chars) -> Result<()> {
    self.lob.erase(offset, count)
  }
}
impl<'lob, 'conn: 'lob> io::Write for ClobWriter<'lob, 'conn> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let (res, piece) = self.lob.impl_.write(self.piece, self.charset, self.lob.form, buf);
    self.piece = piece;
    res
  }
  #[inline]
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
impl<'lob, 'conn: 'lob> Drop for ClobWriter<'lob, 'conn> {
  fn drop(&mut self) {
    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.lob.close(self.piece);//.expect("Error when close CLOB writer");
  }
}
//-------------------------------------------------------------------------------------------------
/// Позволяет читать из большой бинарного объекта в потоковом режиме. Каждый вызов `read` читает очередную порцию данных.
#[derive(Debug)]
pub struct ClobReader<'lob, 'conn: 'lob> {
  lob: &'lob mut Clob<'conn>,
  /// Описательная часть порции данных, получаемых из базы данных (первая или нет).
  piece: Piece,
  /// Кодировка, в которой следует интерпретировать получаемые из базы данных байты.
  charset: Charset,
}
impl<'lob, 'conn: 'lob> ClobReader<'lob, 'conn> {
  /// Получает `CLOB`, читаемый данным читателем.
  pub fn lob(&mut self) -> &mut Clob<'conn> {
    self.lob
  }
}
impl<'lob, 'conn: 'lob> io::Read for ClobReader<'lob, 'conn> {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let (res, piece) = self.lob.impl_.read(self.piece, self.charset, self.lob.form, buf);
    self.piece = piece;
    res
  }
}
impl<'lob, 'conn: 'lob> Drop for ClobReader<'lob, 'conn> {
  fn drop(&mut self) {
    // Невозможно делать панику отсюда, т.к. приложение из-за этого крашится
    let _ = self.lob.close(self.piece);//.expect("Error when close CLOB reader");
  }
}