//! Функции, описанные в разделе [LOB Functions][1] документации Oracle,
//! посвященном работе с большими объектами.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/lob-functions.htm#LNOCI162

use std::io;
use std::os::raw::{c_int, c_void, c_char, c_uchar, c_uint, c_ulonglong, c_ushort};
use std::ptr;

use {Connection, DbResult};
use error::DbError::NeedData;
use types::Charset;

use ffi::DescriptorType;// Типажи для безопасного моста к FFI

use ffi::types;
use ffi::native::{OCIEnv, OCIError, OCISvcCtx};// FFI типы
use ffi::native::misc::{break_, reset};// FFI функции

pub trait OCILobLocator : DescriptorType {}
descriptor!(OCILobLocator, Lob);
descriptor!(OCILobLocator, File);

/// Смысловой номер куска, читаемого из/записываемого в LOB.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LobPiece {
  /// Читаемый/записываемый буфер является единственной частью.
  One   = 0,
  /// Читаемый/записываемый буфер является первой частью набора буферов для чтения/записи.
  First = 1,
  /// Читаемый/записываемый буфер является не первой, но и не последней частью набора буферов
  /// для чтения/записи.
  Next  = 2,
  /// Читаемый/записываемый буфер является последней частью набора буферов для чтения/записи.
  Last  = 3,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharsetForm {
  /// For `CHAR`, `VARCHAR2`, `CLOB` w/o a specified set.
  Implicit = 1,
  /// For `NCHAR`, `NCHAR VARYING`, `NCLOB`.
  NChar    = 2,
  /// For `CHAR`, etc, with `CHARACTER SET ...` syntax.
  Explicit = 3,
  /// For PL/SQL "flexible" parameters.
  Flexible = 4,
  /// For typecheck of `NULL` and `empty_clob()` lits.
  LitNull  = 5,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LobOpenMode {
  /// Readonly mode open for ILOB types.
  ReadOnly      = 1,
  /// Read write mode open for ILOBs.
  ReadWrite     = 2,
  /// Writeonly mode open for ILOB types.
  WriteOnly     = 3,
  /// Appendonly mode open for ILOB types.
  AppendOnly    = 4,
  /// Completely overwrite ILOB.
  FullOverwrite = 5,
  /// Doing a Full Read of ILOB.
  FullRead      = 6,
}
/// Виды временных LOB, которые можно создать вызовом `OCILobCreateTemporary()`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LobType {
  /// Создать временный `BLOB`.
  Blob = 1,
  /// Создать временный `CLOB` или `NCLOB`.
  Clob = 2,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(deprecated)]// Позволяем deprecated внутри перечисления из-за https://github.com/rust-lang/rust/issues/38832
pub enum OCIDuration {
  Process = 5,
  Next = 6,
  UserCallback = 7,
  Default = 8,
  /// The option `OCI_DURATION_NULL` is used when the client does not want to set
  /// the pin duration.  If the object is already loaded into the cache, then the
  /// pin duration will remain the same.  If the object is not yet loaded, the
  /// pin duration of the object will be set to `OCI_DURATION_DEFAULT`.
  Null = 9,
  /// Ресурсы временного LOB автоматически освобождаются при закрытии соединения, которое его породило
  Session = 10,
  /// Ресурсы временного LOB автоматически освобождаются при завершении транзакции, которая его породило
  Trans = 11,
  #[deprecated(note="Not recommented to use by Oracle")]
  Call = 12,
  /// Ресурсы временного LOB автоматически освобождаются при уничтожении выражения, которое его породило
  Statement = 13,
  /// This is to be used only during callouts.  It is similar to that 
  /// of OCI_DURATION_CALL, but lasts only for the duration of a callout.
  /// Its heap is from PGA
  Callout = 14,
  // Ресурсы временного LOB автоматически освобождаются при уничтожении указанного времени жизни.
  // Доступно только в том случае, если окружение было инициализировано в объектном режиме.
  //User(u16),
}
#[derive(Debug)]
pub struct LobImpl<'conn, L: OCILobLocator> {
  conn: &'conn Connection<'conn>,
  locator: *mut L,
}
impl<'conn, L: OCILobLocator> LobImpl<'conn, L> {
  pub fn from(conn: &'conn Connection, locator: *mut L) -> Self {
    LobImpl { conn: conn, locator: locator }
  }
  pub fn temporary_from(conn: &'conn Connection, locator: *mut L, ty: LobType, cache: bool) -> DbResult<Self> {
    let res = unsafe {
      OCILobCreateTemporary(
        conn.context.native_mut(),
        conn.error().native_mut(),
        locator as *mut c_void,
        Charset::Default as u16, // Начиная с Orace 8i требуется передавать значение по умолчанию, которое равно 0
        CharsetForm::Implicit as u8,
        ty as u8,
        cache as c_int,
        OCIDuration::Session as u16
      )
    };
    try!(conn.error().check(res));

    Ok(LobImpl { conn: conn, locator: locator })
  }
  pub fn free_temporary(&self) -> DbResult<()> {
    let res = unsafe {
      OCILobFreeTemporary(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void
      )
    };
    self.conn.error().check(res)
  }
  pub fn is_temporary(&self) -> DbResult<bool> {
    let mut flag = 0;
    let res = unsafe {
      OCILobIsTemporary(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut flag
      )
    };
    try!(self.conn.error().check(res));

    Ok(flag != 0)
  }

  /// Получает количество данных в данном объекте. Для бинарных объектов (`BLOB`-ов) это количество байт,
  /// для символьных (`CLOB`-ов) -- количество символов.
  ///
  /// Данная функция должна вызываться только для не `NULL` LOB-ов, иначе результат не определен.
  pub fn len(&self) -> DbResult<u64> {
    let mut len = 0;
    let res = unsafe {
      OCILobGetLength2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut len
      )
    };
    try!(self.conn.error().check(res));

    Ok(len)
  }
  /// Получает максимально возможный размер для данного большого объекта в байтах.
  pub fn capacity(&self) -> DbResult<u64> {
    let mut capacity = 0;
    let res = unsafe {
      OCILobGetStorageLimit(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut capacity
      )
    };
    try!(self.conn.error().check(res));

    Ok(capacity)
  }
  /// Укорачивает LOB до указанной длины.
  pub fn trim(&mut self, len: u64) -> DbResult<()> {
    let res = unsafe {
      OCILobTrim2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        len
      )
    };
    self.conn.error().check(res)
  }
  pub fn read_impl(&mut self, offset: u64, piece: LobPiece, charset: Charset, buf: &mut [u8], readed: &mut u64) -> DbResult<()> {
    let res = unsafe {
      OCILobRead2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        // Всегда задаем чтение в байтах, даже для [N]CLOB-ов
        readed, ptr::null_mut(),
        // У оракла нумерация с 1, у нас традиционная, с 0
        offset + 1,
        buf.as_mut_ptr() as *mut c_void, buf.len() as u64,
        piece as u8,
        // Функцию обратного вызова не используем
        ptr::null_mut(), None,
        charset as u16, CharsetForm::Implicit as u8
      )
    };

    self.conn.error().check(res)
  }
  pub fn read(&mut self, piece: LobPiece, charset: Charset, buf: &mut [u8]) -> (io::Result<usize>, LobPiece) {
    // Если в прошлый раз при чтении был достигнут конец потока, возвращаем 0
    if piece == LobPiece::Last {
      return (Ok(0), piece);
    }
    let mut readed = 0;
    match self.read_impl(0, piece, charset, buf, &mut readed) {
      Ok(_) => {
        // Чтение закончено, теперь будем постоянно возвращать 0
        (Ok(readed as usize), LobPiece::Last)
      },
      // Не может быть прочитано больше, чем было запрошено, а то, что было запрошено,
      // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
      Err(NeedData) => (Ok(readed as usize), LobPiece::Next),
      Err(e) => (Err(io::Error::new(io::ErrorKind::Other, e)), piece)
    }
  }
  pub fn write_impl(&mut self, offset: u64, piece: LobPiece, charset: Charset, buf: &[u8], writed: &mut u64) -> DbResult<()> {
    let res = unsafe {
      OCILobWrite2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        // Всегда задаем запись в байтах, даже для [N]CLOB-ов
        writed, ptr::null_mut(),
        // У оракла нумерация с 1, у нас традиционная, с 0
        // Имеет значение только при первом вызове, при последующих игнорируется
        offset + 1,
        buf.as_ptr() as *mut c_void, buf.len() as u64,
        piece as u8,
        // Функцию обратного вызова не используем
        ptr::null_mut(), None,
        // Данные параметры игнорируются для BLOB-ов.
        charset as u16, CharsetForm::Implicit as u8
      )
    };

    self.conn.error().check(res)
  }
  pub fn write(&mut self, piece: LobPiece, charset: Charset, buf: &[u8]) -> (io::Result<usize>, LobPiece) {
    // Если в прошлый раз при записи был достигнут конец потока, возвращаем 0
    if piece == LobPiece::Last {
      return (Ok(0), piece);
    }
    let mut writed = 0;
    match self.write_impl(0, piece, charset, buf, &mut writed) {
      Ok(_) => {
        // Чтение закончено, теперь будем постоянно возвращать 0
        (Ok(writed as usize), LobPiece::Last)
      },
      // Не может быть записано больше, чем было запрошено, а то, что было запрошено,
      // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
      Err(NeedData) => (Ok(writed as usize), LobPiece::Next),
      Err(e) => (Err(io::Error::new(io::ErrorKind::Other, e)), piece)
    }
  }
  /// Дописывает в конец данного LOB-а данные из указанного буфера.
  pub fn append(&mut self, piece: LobPiece, charset: Charset, buf: &[u8]) -> DbResult<usize> {
    // Количество того, сколько писать и сколько было реально записано
    let mut writed = buf.len() as u64;
    let res = unsafe {
      OCILobWriteAppend2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        // Всегда задаем запись в байтах, даже для [N]CLOB-ов
        &mut writed, ptr::null_mut(),
        buf.as_ptr() as *mut c_void, buf.len() as u64,
        piece as u8,
        // Функцию обратного вызова не используем
        ptr::null_mut(), None,
        // Данные параметры игнорируются для BLOB-ов.
        charset as u16, CharsetForm::Implicit as u8
      )
    };
    try!(self.conn.error().check(res));

    // Не может быть записано больше, чем было запрошено, а то, что было запрошено,
    // не превышает usize, поэтому приведение безопасно в случае, если sizeof(usize) < sizeof(u64).
    Ok(writed as usize)
  }
  /// Заполняет LOB, начиная с указанного индекса, указанным количеством нулей (для бинарных данных) или
  /// пробелов (для символьных данных). После завершения работы в `count` будет записано реальное количество
  /// очищенных символов/байт.
  pub fn erase(&mut self, offset: u64, count: &mut u64) -> DbResult<()> {
    let res = unsafe {
      OCILobErase2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        count, offset + 1// У оракла нумерация с 1, у нас традиционная, с 0
      )
    };
    self.conn.error().check(res)
  }
  /// Добавляет в конец содержимого данного LOB-а содержимое другого LOB-а. Оба должны
  /// иметь один и тот же тип и быть внутренними LOB-оми, а не файловыми LOB-ами.
  pub fn add(&mut self, src: &LobImpl<L>) -> DbResult<()> {
    let res = unsafe {
      OCILobAppend(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        src.locator as *mut c_void,
      )
    };
    self.conn.error().check(res)
  }
  pub fn open(&mut self, mode: LobOpenMode) -> DbResult<()> {
    let res = unsafe {
      OCILobOpen(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        mode as u8
      )
    };
    self.conn.error().check(res)
  }
  pub fn close(&mut self) -> DbResult<()> {
    let res = unsafe {
      OCILobClose(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void
      )
    };
    self.conn.error().check(res)
  }
  pub fn is_open(&self) -> DbResult<bool> {
    let mut flag = 0;
    let res = unsafe {
      OCILobIsOpen(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut flag
      )
    };
    try!(self.conn.error().check(res));

    Ok(flag != 0)
  }
  pub fn try_eq(&self, other: &Self) -> DbResult<bool> {
    let env = self.conn.get_env();
    let mut flag = 0;
    let res = unsafe {
      OCILobIsEqual(
        env.native() as *mut OCIEnv,
        self.locator as *const c_void,
        other.locator as *const c_void,
        &mut flag
      )
    };
    try!(env.error().check(res));

    Ok(flag != 0)
  }
  pub fn break_(&mut self) -> DbResult<()> {
    break_(&self.conn.context, self.conn.error())
  }
  pub fn reset(&mut self) -> DbResult<()> {
    reset(&self.conn.context, self.conn.error())
  }
}
impl<'conn> LobImpl<'conn, Lob> {
  pub fn get_chunk_size(&self) -> DbResult<u32> {
    let mut size = 0;
    let res = unsafe {
      OCILobGetChunkSize(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut size
      )
    };
    try!(self.conn.error().check(res));

    Ok(size)
  }
  /// Получает кодировку базы данных для данного большого символьного объекта.
  pub fn charset(&self) -> DbResult<Charset> {
    let env = self.conn.get_env();
    let mut charset = Charset::Default;
    let res = unsafe {
      OCILobCharSetId(
        env.native() as *mut OCIEnv,
        self.conn.error().native_mut(),
        self.locator as *const c_void,
        &mut charset as *mut Charset as *mut u16
      )
    };
    try!(self.conn.error().check(res));

    Ok(charset)
  }
}
impl<'conn> LobImpl<'conn, File> {
  pub fn set_filename(&mut self, directory: &str, filename: &str) -> DbResult<()> {
    let res = unsafe {
      OCILobFileSetName(
        ptr::null_mut(),//self.conn.server.env.env.native as *mut OCIEnv,
        self.conn.error().native_mut(),
        &mut self.locator as *mut *mut File as *mut *mut c_void,
        //FIXME: Данные строки могут быть в UTF-16, если при вызове OCIEnvNlsCreate() использовалась она
        // Длина срезов возвращается, как и положено, в байтах
        directory.as_ptr() as *const c_char, directory.len() as u16,
        filename.as_ptr()  as *const c_char, filename.len()  as u16
      )
    };
    self.conn.error().check(res)
  }
  /// Проверяет, что указанный файл с данными существует на файловой системе сервера базы данных.
  pub fn is_exist(&self) -> DbResult<bool> {
    let mut flag = 0;
    let res = unsafe {
      OCILobFileExists(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator as *mut c_void,
        &mut flag
      )
    };
    try!(self.conn.error().check(res));

    Ok(flag != 0)
  }
}
impl<'conn, L: OCILobLocator> PartialEq for LobImpl<'conn, L> {
  fn eq(&self, other: &Self) -> bool {
    self.try_eq(other).expect("Error when compare LOB")
  }
}
impl<'conn, L: OCILobLocator> Eq for LobImpl<'conn, L> {}

/// The callback function must return `OCI_CONTINUE` for the read to continue. If any other error code is returned,
/// the LOB read is terminated.
///
/// # Параметры
/// - ctxp (IN):
///   The context for the callback function. Can be `NULL`.
/// - bufp (IN/OUT):
///   A buffer pointer for the piece.
/// - lenp (IN):
///   The length in bytes of the current piece in `bufp`.
/// - piecep (IN)
///   Which piece: `OCI_FIRST_PIECE`, `OCI_NEXT_PIECE`, or `OCI_LAST_PIECE`.
/// - changed_bufpp (OUT):
///   The callback function can put the address of a new buffer if it prefers to use a new buffer for the next piece
///   to read. The default old buffer `bufp` is used if this parameter is set to `NULL`.
/// - changed_lenp (OUT):
///   Length of the new buffer, if provided.
pub type OCICallbackLobRead2  = extern "C" fn(ctxp: *mut c_void,
                                              bufp: *const c_void,
                                              lenp: u64,
                                              piecep: u8,
                                              changed_bufpp: *mut *mut c_void,
                                              changed_lenp: *mut u64);
pub type OCICallbackLobWrite2 = extern "C" fn(ctxp: *mut c_void,
                                              bufp: *mut c_void,
                                              lenp: *mut u64,
                                              piecep: *mut u8,
                                              changed_bufpp: *mut *mut c_void,
                                              changed_lenp: *mut u64);

// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Starts a user duration for a temporary LOB.
  pub fn OCIDurationBegin(env: *mut OCIEnv,
                          err: *mut OCIError,
                          svc: *const OCISvcCtx,
                          parent: u16,
                          duration: *mut u16// результат
                          ) -> c_int;
  /// Terminates a user duration for a temporary LOB.
  pub fn OCIDurationEnd(env: *mut OCIEnv,
                        err: *mut OCIError,
                        svc: *const OCISvcCtx,
                        duration: u16) -> c_int;

  /// Appends a LOB value at the end of another LOB as specified.
  fn OCILobAppend(svchp: *mut OCISvcCtx,
                  errhp: *mut OCIError,
                  // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                  dst_locp: *mut c_void/*OCILobLocator*/,
                  src_locp: *mut c_void/*OCILobLocator*/) -> c_int;

  /// Reads LOB data for multiple locators in one round-trip.
  /// This function can be used for LOBs of size greater than or less than 4 GB.
  pub fn OCILobArrayRead(svchp: *mut OCISvcCtx,
                         errhp: *mut OCIError,
                         array_iter: *mut c_uint,
                         // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                         locp_arr: *mut *mut c_void/*OCILobLocator*/,
                         byte_amt_arr: *mut c_ulonglong,
                         char_amt_arr: *mut c_ulonglong,
                         offset_arr: *mut c_ulonglong,
                         bufp_arr: *mut *mut c_void,
                         bufl_arr: c_ulonglong,
                         piece: c_uchar,
                         ctxp: *mut c_void,
                         cbfp: Option<types::OCICallbackLobArrayRead>,
                         csid: c_ushort,
                         csfrm: c_uchar) -> c_int;
  /// Writes LOB data for multiple locators in one round-trip.
  /// This function can be used for LOBs of size greater than or less than 4 GB.
  pub fn OCILobArrayWrite(svchp: *mut OCISvcCtx,
                          errhp: *mut OCIError,
                          array_iter: *mut c_uint,
                          // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                          locp_arr: *mut *mut c_void/*OCILobLocator*/,
                          byte_amt_arr: *mut c_ulonglong,
                          char_amt_arr: *mut c_ulonglong,
                          offset_arr: *mut c_ulonglong,
                          bufp_arr: *mut *mut c_void,
                          bufl_arr: *mut c_ulonglong,
                          piece: c_uchar,
                          ctxp: *mut c_void,
                          cbfp: Option<types::OCICallbackLobArrayWrite>,
                          csid: c_ushort,
                          csfrm: c_uchar) -> c_int;

  /// Assigns one LOB or BFILE locator to another.
  fn OCILobLocatorAssign(svchp: *mut OCISvcCtx,
                         errhp: *mut OCIError,
                         // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                         src_locp: *const c_void/*OCILobLocator*/,
                         dst_locpp: *mut *mut c_void/*OCILobLocator*/) -> c_int;

  /// Gets the length of a LOB. This function must be used for LOBs of size greater than 4 GB. You can also use this
  /// function for LOBs smaller than 4 GB.
  ///
  /// Gets the length of a LOB. If the LOB is NULL, the length is undefined. The length of a `BFILE` includes the EOF,
  /// if it exists. The length of an empty internal LOB is zero.
  ///
  /// Regardless of whether the client-side character set is varying-width, the output length is in characters for
  /// `CLOB`s and `NCLOB`s, and in bytes for `BLOB`s and `BFILE`s.
  ///
  /// Note:
  ///
  /// > Any zero-byte or space fillers in the LOB written by previous calls to `OCILobErase2()` or `OCILobWrite2()` are also
  /// > included in the length count.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN):
  ///   A LOB locator that uniquely references the LOB. For internal LOBs, this locator must have been a locator
  ///   that was obtained from the server specified by `svchp`. For BFILEs, the locator can be set by `OCILobFileSetName()`,
  ///   by a `SELECT` statement, or by `OCIObjectPin()`.
  /// - lenp (OUT):
  ///   On output, it is the length of the LOB if the LOB is not NULL. For character LOBs, it is the number of characters;
  ///   for binary LOBs and `BFILE`s, it is the number of bytes in the LOB.
  fn OCILobGetLength2(svchp: *mut OCISvcCtx,
                      errhp: *mut OCIError,
                      // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                      locp: *mut c_void/*OCILobLocator*/,
                      lenp: *mut u64) -> c_int;
  /// Gets the maximum length of an internal LOB (BLOB, CLOB, or NCLOB) in bytes.
  ///
  /// Because block size ranges from 2 KB to 32 KB, the maximum LOB size ranges from 8 terabytes to 128 terabytes (TB) for LOBs.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN):
  ///   A LOB locator that uniquely references the LOB. The locator must have been one that was obtained from the server
  ///   specified by `svchp`.
  /// - limitp (OUT):
  ///   The maximum length of the LOB (in bytes) that can be stored in the database.
  fn OCILobGetStorageLimit(svchp: *mut OCISvcCtx,
                           errhp: *mut OCIError,
                           // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                           locp: *mut c_void/*OCILobLocator*/,
                           limitp: *mut u64) -> c_int;

  /// Reads a portion of a LOB or `BFILE`, as specified by the call, into a buffer. This function must be used
  /// for LOBs of size greater than 4 GB. You can also use this function for LOBs smaller than 4 GB.
  ///
  /// Reads a portion of a LOB or `BFILE` as specified by the call into a buffer. It is an error to try to read
  /// from a `NULL` LOB or `BFILE`.
  ///
  /// Note:
  /// > When you read or write LOBs, specify a character set form (`csfrm`) that matches the form of the locator itself.
  ///
  /// For BFILEs, the operating system file must exist on the server, and it must have been opened by `OCILobFileOpen()`
  /// or `OCILobOpen()` using the input locator. Oracle Database must have permission to read the operating system file,
  /// and the user must have read permission on the directory object.
  ///
  /// When you use the polling mode for `OCILobRead2()`, the first call must specify values for `offset` and `amtp`, but
  /// on subsequent polling calls to `OCILobRead2()`, you need not specify these values.
  ///
  /// If the LOB is a `BLOB`, the `csid` and `csfrm` parameters are ignored.
  ///
  /// Note:
  ///
  /// > To terminate an `OCILobRead2()` operation and free the statement handle, use the `OCIBreak()` call.
  ///
  /// The following points apply to reading LOB data in streaming mode:
  ///
  /// * When you use polling mode, be sure to specify the `char_amtp` and `byte_amtp` and `offset` parameters only in the
  ///   first call to `OCILobRead2()`. On subsequent polling calls these parameters are ignored. If both `byte_amtp` and
  ///   `char_amtp` are set to point to zero and `OCI_FIRST_PIECE` is passed, then polling mode is assumed and data is
  ///   read till the end of the LOB. On output, `byte_amtp` gives the number of bytes read in the current piece. For `CLOB`s
  ///   and `NCLOB`s, `char_amtp` gives the number of characters read in the current piece.
  /// * When you use callbacks, the `len` parameter, which is input to the callback, indicates how many bytes are filled
  ///   in the buffer. Check the `len` parameter during your callback processing, because the entire buffer cannot be
  ///   filled with data.
  /// * When you use polling, look at the `byte_amtp` parameter to see how much the buffer is filled for the current piece.
  ///   For `CLOB`s and `NCLOB`s, `char_amtp` returns the number of characters read in the buffer as well.
  ///
  /// To read data in UTF-16 format, set the `csid` parameter to `OCI_UTF16ID`. If the `csid` parameter is set, it overrides
  /// the `NLS_LANG` environment variable.
  ///
  /// # Parameters
  /// - svchp (IN/OUT):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN):
  ///   A LOB or `BFILE` locator that uniquely references the LOB or `BFILE`. This locator must have been a locator that
  ///   was obtained from the server specified by `svchp`.
  /// - byte_amtp (IN/OUT):
  ///   * IN - The number of bytes to read from the database. Used for `BLOB` and `BFILE` always. For `CLOB` and `NCLOB`,
  ///     it is used only when `char_amtp` is zero.
  ///   * OUT - The number of bytes read into the user buffer.
  /// - char_amtp (IN/OUT):
  ///   * IN - The maximum number of characters to read into the user buffer. Ignored for `BLOB` and `BFILE`.
  ///   * OUT - The number of characters read into the user buffer. Undefined for `BLOB` and `BFILE`.
  /// - offset (IN):
  ///   On input, this is the absolute offset from the beginning of the LOB value. For character LOBs (`CLOB`s, `NCLOB`s),
  ///   it is the number of characters from the beginning of the LOB; for binary LOBs or `BFILE`s, it is the number of
  ///   bytes. The first position is `1`.
  ///
  ///   If you use streaming (by polling or a callback), specify the offset in the first call; in subsequent polling calls,
  ///   the offset parameter is ignored. When you use a callback, there is no offset parameter.
  /// - bufp (IN/OUT):
  ///   The pointer to a buffer into which the piece is read. The length of the allocated memory is assumed to be `bufl`.
  /// - bufl (IN):
  ///   The length of the buffer in octets. This value differs from the `amtp` value for `CLOB`s and for `NCLOB`s
  ///   (`csfrm=SQLCS_NCHAR`) when the `amtp` parameter is specified in terms of characters, and the `bufl` parameter is
  ///   specified in terms of bytes.
  /// - piece (IN):
  ///   `OCI_ONE_PIECE` - The call never assumes polling. If the amount indicated is more than the buffer length, then the
  ///   buffer is filled as much as possible.
  ///
  ///   For polling, pass `OCI_FIRST_PIECE` the first time and `OCI_NEXT_PIECE` in subsequent calls. `OCI_FIRST_PIECE` should
  ///   be passed while using the callback.
  /// - ctxp (IN):
  ///   The context pointer for the callback function. Can be `NULL`.
  /// - cbfp (IN):
  ///   A callback that can be registered to be called for each piece. If this is `NULL`, then `OCI_NEED_DATA` is returned
  ///   for each piece.
  /// - csid (IN):
  ///   The character set ID of the buffer data. If this value is `0`, then `csid` is set to the client's `NLS_LANG` or
  ///   `NLS_CHAR` value, depending on the value of `csfrm`. It is never assumed to be the server character set, unless
  ///   the server and client have the same settings.
  /// - csfrm (IN):
  ///   The character set form of the buffer data. The `csfrm` parameter must be consistent with the type of the LOB.
  ///
  ///   The `csfrm` parameter has two possible nonzero values:
  ///
  ///   * `SQLCS_IMPLICIT` - Database character set ID
  ///   * `SQLCS_NCHAR` - `NCHAR` character set ID
  ///
  ///   The default value is `SQLCS_IMPLICIT`. If `csfrm` is not specified, the default is assumed.
  fn OCILobRead2(svchp: *mut OCISvcCtx,
                 errhp: *mut OCIError,
                 // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                 locp: *mut c_void/*OCILobLocator*/,
                 byte_amtp: *mut u64,
                 char_amtp: *mut u64,
                 offset: u64,
                 bufp: *mut c_void,
                 bufl: u64,
                 piece: u8,
                 ctxp: *mut c_void,
                 cbfp: Option<OCICallbackLobRead2>,
                 csid: u16,
                 csfrm: u8) -> c_int;
  /// Writes a buffer into a LOB. This function must be used for LOBs of size greater than 4 GB.
  /// You can also use this function for LOBs smaller than 4 GB.
  fn OCILobWrite2(svchp: *mut OCISvcCtx,
                  errhp: *mut OCIError,
                  // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                  locp: *mut c_void/*OCILobLocator*/,
                  byte_amtp: *mut u64,
                  char_amtp: *mut u64,
                  offset: u64,
                  bufp: *mut c_void,
                  buflen: u64,
                  piece: u8,
                  ctxp: *mut c_void,
                  cbfp: Option<OCICallbackLobWrite2>,
                  csid: u16,
                  csfrm: u8) -> c_int;
  /// Writes data starting at the end of a LOB. This function must be used for LOBs of size greater than 4 GB. You can also
  /// use this function for LOBs smaller than 4 GB.
  ///
  /// The buffer can be written to the LOB in a single piece with this call, or it can be provided piecewise using callbacks
  /// or a standard polling method. If the value of the piece parameter is `OCI_FIRST_PIECE`, data must be provided through
  /// callbacks or polling. If a callback function is defined in the `cbfp` parameter, then this callback function is invoked
  /// to get the next piece after a piece is written to the pipe. Each piece is written from bufp. If no callback function
  /// is defined, then `OCILobWriteAppend2()` returns the `OCI_NEED_DATA` error code.
  ///
  /// The application must call `OCILobWriteAppend2()` again to write more pieces of the LOB. In this mode, the buffer pointer
  /// and the length can be different in each call if the pieces are of different sizes and from different locations. A piece
  /// value of `OCI_LAST_PIECE` terminates the piecewise write.
  ///
  /// The `OCILobWriteAppend2()` function is not supported if LOB buffering is enabled.
  ///
  /// If the LOB is a `BLOB`, the csid and csfrm parameters are ignored.
  ///
  /// If both `byte_amtp` and `char_amtp` are set to point to zero amount and `OCI_FIRST_PIECE` is given as input, then polling
  /// mode is assumed and data is written until you specify `OCI_LAST_PIECE`. For `CLOB`s and `NCLOB`s, `byte_amtp` and
  /// `char_amtp` return the data written by each piece in terms of number of bytes and number of characters respectively.
  /// For `BLOB`s, `byte_amtp` returns the number of bytes written by each piece whereas `char_amtp` is undefined on output.
  ///
  /// It is not mandatory that you wrap this LOB operation inside the open or close calls. If you did not open the LOB before
  /// performing this operation, then the functional and domain indexes on the LOB column are updated during this call. However,
  /// if you did open the LOB before performing this operation, then you must close it before you commit your transaction.
  /// When an internal LOB is closed, it updates the functional and domain indexes on the LOB column.
  ///
  /// If you do not wrap your LOB operations inside the open or close API, then the functional and domain indexes are updated
  /// each time you write to the LOB. This can adversely affect performance. If you have functional or domain indexes, Oracle
  /// recommends that you enclose write operations to the LOB within the open or close statements.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   An internal LOB locator that uniquely references a LOB.
  /// - byte_amtp (IN/OUT):
  ///   * IN - The number of bytes to write to the database. Used for `BLOB`. For `CLOB` and `NCLOB` it is used only when
  ///     `char_amtp` is zero.
  ///   * OUT - The number of bytes written to the database.
  /// - char_amtp (IN/OUT):
  ///   * IN - The maximum number of characters to write to the database. Ignored for `BLOB`.
  ///   * OUT - The number of characters written to the database. Undefined for `BLOB`.
  /// - bufp (IN):
  ///   The pointer to a buffer from which the piece is written. The length of the data in the buffer is assumed to be the
  ///   value passed in buflen. Even if the data is being written in pieces, bufp must contain the first piece of the LOB
  ///   when this call is invoked. If a callback is provided, `bufp` must not be used to provide data or an error results.
  /// - buflen (IN):
  ///   The length, in bytes, of the data in the buffer. Note that this parameter assumes an 8-bit byte. If your operating
  ///   system uses a longer byte, the value of buflen must be adjusted accordingly.
  /// - piece (IN):
  ///   Which piece of the buffer is being written. The default value for this parameter is `OCI_ONE_PIECE`, indicating that
  ///   the buffer is written in a single piece. The following other values are also possible for piecewise or callback mode:
  ///   `OCI_FIRST_PIECE`, `OCI_NEXT_PIECE`, and `OCI_LAST_PIECE`.
  /// - ctxp (IN):
  ///   The context for the callback function. Can be `NULL`.
  /// - cbfp (IN):
  ///   A callback that can be registered to be called for each piece in a piecewise write. If this is `NULL`, the standard
  ///   polling method is used. The callback function must return `OCI_CONTINUE` for the write to continue. If any other
  ///   error code is returned, the LOB write is terminated. The callback takes the following parameters:
  /// - csid (IN):
  ///   The character set ID of the buffer data.
  /// - csfrm (IN):
  ///   The character set form of the buffer data.
  ///
  ///   The `csfrm` parameter has two possible nonzero values:
  ///   * `SQLCS_IMPLICIT` - Database character set ID
  ///   * `SQLCS_NCHAR` - NCHAR character set ID
  ///
  ///   The default value is `SQLCS_IMPLICIT`.
  fn OCILobWriteAppend2(svchp: *mut OCISvcCtx,
                        errhp: *mut OCIError,
                        // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                        locp: *mut c_void/*OCILobLocator*/,
                        byte_amtp: *mut u64,
                        char_amtp: *mut u64,
                        bufp: *mut c_void,
                        buflen: u64,
                        piece: u8,
                        ctxp: *mut c_void,
                        cbfp: Option<OCICallbackLobWrite2>,
                        csid: u16,
                        csfrm: u8) -> c_int;

  /// Erases a specified portion of the internal LOB data starting at a specified offset. This function must be used for LOBs
  /// of size greater than 4 GB. You can also use this function for LOBs smaller than 4 GB.
  ///
  /// The actual number of characters or bytes erased is returned. For `BLOB`s, erasing means that zero-byte fillers overwrite
  /// the existing LOB value. For CLOBs, erasing means that spaces overwrite the existing LOB value.
  ///
  /// This function is valid only for internal LOBs; BFILEs are not allowed.
  ///
  /// It is not mandatory that you wrap this LOB operation inside the open or close calls. If you did not open the LOB before
  /// performing this operation, then the functional and domain indexes on the LOB column are updated during this call. However,
  /// if you did open the LOB before performing this operation, then you must close it before you commit your transaction. When
  /// an internal LOB is closed, it updates the functional and domain indexes on the LOB column.
  ///
  /// If you do not wrap your LOB operations inside the open or close API, then the functional and domain indexes are updated
  /// each time you write to the LOB. This can adversely affect performance. If you have functional or domain indexes, Oracle
  /// recommends that you enclose write operations to the LOB within the open or close statements.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to OCIErrorGet() for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   An internal LOB locator that uniquely references the LOB. This locator must have been a locator that was obtained
  ///   from the server specified by `svchp`.
  /// - amount (IN/OUT):
  ///   The number of characters for CLOBs or NCLOBs, or bytes for BLOBs, to erase. On IN, the value signifies the number
  ///   of characters or bytes to erase. On OUT, the value identifies the actual number of characters or bytes erased.
  /// - offset (IN):
  ///   Absolute offset in characters for CLOBs or NCLOBs, or bytes for BLOBs, from the beginning of the LOB value from which
  ///   to start erasing data. Starts at `1`.
  fn OCILobErase2(svchp: *mut OCISvcCtx,
                  errhp: *mut OCIError,
                  // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                  locp: *mut c_void/*OCILobLocator*/,
                  amount: *mut u64,
                  offset: u64) -> c_int;
  /// Truncates the LOB value to a shorter length. This function must be used for LOBs of size greater than 4 GB.
  /// You can also use this function for LOBs smaller than 4 GB.
  ///
  /// This function trims the LOB data to a specified shorter length. The function returns an error if newlen is
  /// greater than the current LOB length. This function is valid only for internal LOBs. `BFILE`s are not allowed.
  ///
  /// It is not mandatory that you wrap this LOB operation inside the open or close calls. If you did not open the
  /// LOB before performing this operation, then the functional and domain indexes on the LOB column are updated
  /// during this call. However, if you did open the LOB before performing this operation, then you must close it
  /// before you commit your transaction. When an internal LOB is closed, it updates the functional and domain
  /// indexes on the LOB column.
  ///
  /// If you do not wrap your LOB operations inside the open or close API, then the functional and domain indexes
  /// are updated each time you write to the LOB. This can adversely affect performance. If you have functional or
  /// domain indexes, Oracle recommends that you enclose write operations to the LOB within the open or close statements.
  ///
  /// # Parameters
  ///
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to OCIErrorGet() for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   An internal LOB locator that uniquely references the LOB. This locator must have been a locator that was
  ///   obtained from the server specified by `svchp`.
  /// - newlen (IN):
  ///   The new length of the LOB value, which must be less than or equal to the current length. For character LOBs,
  ///   it is the number of characters; for binary LOBs and `BFILE`s, it is the number of bytes in the LOB.
  fn OCILobTrim2(svchp: *mut OCISvcCtx,
                 errhp: *mut OCIError,
                 // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                 locp: *mut c_void/*OCILobLocator*/,
                 newlen: u64) -> c_int;

  /// Opens a LOB, internal or external, in the indicated mode.
  ///
  /// It is an error to open the same LOB twice. BFILEs cannot be opened in read/write mode. If a user tries to write to
  /// a LOB or BFILE that was opened in read-only mode, an error is returned.
  ///
  /// Opening a LOB requires a round-trip to the server for both internal and external LOBs. For internal LOBs, the open
  /// triggers other code that relies on the open call. For external LOBs (BFILEs), open requires a round-trip because
  /// the actual operating system file on the server side is being opened.
  ///
  /// It is not necessary to open a LOB to perform operations on it. When using function-based indexes, extensible indexes
  /// or context, and making multiple calls to update or write to the LOB, you should first call `OCILobOpen()`, then update
  /// the LOB as many times as you want, and finally call `OCILobClose()`. This sequence of operations ensures that the
  /// indexes are only updated once at the end of all the write operations instead of once for each write operation.
  ///
  /// It is not mandatory that you wrap all LOB operations inside the open and close calls. However, if you open a LOB,
  /// then you must close it before you commit your transaction. When an internal LOB is closed, it updates the functional
  /// and domain indexes on the LOB column. It is an error to commit the transaction before closing all opened LOBs that
  /// were opened by the transaction.
  ///
  /// When the error is returned, the LOB is no longer marked as open, but the transaction is successfully committed.
  /// Hence, all the changes made to the LOB and non-LOB data in the transaction are committed, but the domain and
  /// function-based indexing are not updated. If this happens, rebuild your functional and domain indexes on the LOB column.
  ///
  /// If you do not wrap your LOB operations inside the open or close API, then the functional and domain indexes are updated
  /// each time you write to the LOB. This can adversely affect performance, so if you have functional or domain indexes, Oracle
  /// recommends that you enclose write operations to the LOB within the open or close statements.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   The LOB to open. The locator can refer to an internal or external LOB.
  /// - mode (IN):
  ///   The mode in which to open the LOB or BFILE. In Oracle8i or later, valid modes for LOBs are `OCI_LOB_READONLY`
  ///   and `OCI_LOB_READWRITE`. Note that `OCI_FILE_READONLY` exists as input to `OCILobFileOpen()`. `OCI_FILE_READONLY`
  ///   can be used with `OCILobOpen()` if the input locator is for a BFILE.
  fn OCILobOpen(svchp: *mut OCISvcCtx,
                errhp: *mut OCIError,
                // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                locp: *mut c_void/*OCILobLocator*/,
                mode: u8) -> c_int;

  /// Closes a previously opened LOB or BFILE.
  ///
  /// Closes a previously opened internal or external LOB. No error is returned if the BFILE exists but is not
  /// opened. An error is returned if the internal LOB is not open.
  ///
  /// Closing a LOB requires a round-trip to the server for both internal and external LOBs. For internal LOBs,
  /// close triggers other code that relies on the close call and for external LOBs (BFILEs), close actually
  /// closes the server-side operating system file.
  ///
  /// It is not mandatory that you wrap all LOB operations inside the open or close calls. However, if you open
  /// a LOB, then you must close it before you commit your transaction. When an internal LOB is closed, it updates
  /// the functional and domain indexes on the LOB column. It is an error to commit the transaction before closing
  /// all opened LOBs that were opened by the transaction.
  ///
  /// When the error is returned, the LOB is no longer marked as open, but the transaction is successfully committed.
  /// Hence, all the changes made to the LOB and non-LOB data in the transaction are committed, but the domain and
  /// function-based indexing are not updated. If this happens, rebuild your functional and domain indexes on the LOB
  /// column.
  ///
  /// If you do not wrap your LOB operations inside the open or close API, then the functional and domain indexes are
  /// updated each time you write to the LOB. This can adversely affect performance, so if you have functional or domain
  /// indexes, Oracle recommends that you enclose write operations to the LOB within the open or close statements.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to OCIErrorGet() for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   The LOB to close. The locator can refer to an internal or external LOB.
  fn OCILobClose(svchp: *mut OCISvcCtx,
                 errhp: *mut OCIError,
                 // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                 locp: *mut c_void/*OCILobLocator*/) -> c_int;
  /// Tests whether a LOB or `BFILE` is open.
  ///
  /// Checks to see if the internal LOB is open or if the `BFILE` was opened using the input locator.
  ///
  /// * For `BFILE`s
  ///   If the input `BFILE` locator was never passed to `OCILobOpen()` or `OCILobFileOpen()`, the `BFILE` is considered not to be opened
  ///   by this `BFILE` locator. However, a different `BFILE` locator may have opened the `BFILE`. Multiple opens can be performed on the
  ///   same `BFILE` using different locators. In other words, openness is associated with a specific locator for `BFILE`s.
  /// * For internal LOBs
  ///     Openness is associated with the LOB, not with the locator. If locator1 opened the LOB, then locator2 also sees the LOB as open.
  ///
  /// For internal LOBs, this call requires a server round-trip because it checks the state on the server to see if the LOB is open.
  /// For external LOBs (`BFILE`s), this call also requires a round-trip because the operating system file on the server side must be
  /// checked to see if it is open.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that can be passed to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN):
  ///   Pointer to the LOB locator being examined. The locator can refer to an internal or external LOB.
  /// - flag (OUT):
  ///   Returns `TRUE` if the internal LOB is open or if the `BFILE` was opened using the input locator. Returns FALSE if it was not.
  fn OCILobIsOpen(svchp: *mut OCISvcCtx,
                  errhp: *mut OCIError,
                  // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                  locp: *mut c_void/*OCILobLocator*/,
                  flag: *mut c_int) -> c_int;

  /// Creates a temporary LOB.
  ///
  /// This function creates a temporary LOB and its corresponding index in the user's temporary tablespace.
  ///
  /// When this function is complete, the locp parameter points to an empty temporary LOB whose length is zero.
  ///
  /// The lifetime of the temporary LOB is determined by the duration parameter. At the end of its duration the
  /// temporary LOB is freed. An application can free a temporary LOB sooner with the `OCILobFreeTemporary()` call.
  ///
  /// If the LOB is a `BLOB`, the `csid` and `csfrm` parameters are ignored.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The OCI service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   A locator that points to the temporary LOB. You must allocate the locator using `OCIDescriptorAlloc()` before
  ///   passing it to this function. It does not matter whether this locator points to a LOB; the temporary LOB gets
  ///   overwritten either way.
  /// - csid (IN):
  ///   The LOB character set ID. For Oracle8i or later, pass as `OCI_DEFAULT`.
  /// - csfrm (IN):
  ///   The LOB character set form of the buffer data. The csfrm parameter has two possible nonzero values:
  ///   * SQLCS_IMPLICIT - Database character set ID, to create a `CLOB`. `OCI_DEFAULT` can also be used to implicitly
  ///     create a `CLOB`.
  ///   * `SQLCS_NCHAR` - `NCHAR` character set ID, to create an `NCLOB`.
  ///
  ///   The default value is `SQLCS_IMPLICIT`.
  /// - lobtype (IN):
  ///   The type of LOB to create. Valid values include:
  ///   * `OCI_TEMP_BLOB` - For a temporary `BLOB`
  ///   * `OCI_TEMP_CLOB` - For a temporary `CLOB` or `NCLOB`
  /// - cache (IN):
  ///   Pass `TRUE` if the temporary LOB should be read into the cache; pass `FALSE` if it should not. The default
  ///   is `FALSE` for `NOCACHE` functionality.
  /// - duration (IN):
  ///   The duration of the temporary LOB. The following are valid values:
  ///   * `OCI_DURATION_SESSION`
  ///   * `OCI_DURATION_CALL`
  fn OCILobCreateTemporary(svchp: *mut OCISvcCtx,
                           errhp: *mut OCIError,
                           // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                           locp: *mut c_void/*OCILobLocator*/,
                           csid: u16,
                           csfrm: u8,
                           lobtype: u8,
                           cache: c_int,
                           duration: u16) -> c_int;
  /// Frees a temporary LOB.
  ///
  /// This function frees the contents of the temporary LOB to which this locator points. Note that the locator
  /// itself is not freed until `OCIDescriptorFree()` is called. You must always call `OCILobFreeTemporary()` before
  /// calling `OCIDescriptorFree()` or `OCIArrayDescriptorFree()` to free the contents of the temporary LOB. See
  /// [About Freeing Temporary LOBs][1] for more information.
  ///
  /// This function returns an error if the LOB locator passed in the `locp` parameter does not point to a temporary
  /// LOB, possibly because the LOB locator:
  /// * Points to a permanent LOB
  /// * Pointed to a temporary LOB that has been freed
  /// * Has never pointed to anything
  ///
  /// # Parameters
  /// - svchp (IN/OUT):
  ///   The OCI service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to OCIErrorGet() for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   A locator uniquely referencing the LOB to be freed.
  ///
  /// [1]: http://docs.oracle.com/database/122/LNOCI/lobs-and-bfile-operations.htm#GUID-19F5922C-3560-476B-B414-27F13B5C2BAC
  fn OCILobFreeTemporary(svchp: *mut OCISvcCtx,
                         errhp: *mut OCIError,
                         // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                         locp: *mut c_void/*OCILobLocator*/) -> c_int;
  /// Tests if a locator points to a temporary LOB
  ///
  /// This function tests a locator to determine if it points to a temporary LOB. If so, is_temporary is set to `TRUE`.
  /// If not, is_temporary is set to `FALSE`.
  ///
  /// # Parameters
  /// - envhp (IN):
  ///   The OCI environment handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN):
  ///   The locator to test.
  /// - is_temporary (OUT):
  ///   Returns TRUE if the LOB locator points to a temporary LOB; `FALSE` if it does not.
  fn OCILobIsTemporary(svchp: *mut OCISvcCtx,
                       errhp: *mut OCIError,
                       // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                       locp: *mut c_void/*OCILobLocator*/,
                       is_temporary: *mut c_int) -> c_int;

  /// Gets the chunk size of a LOB.
  ///
  /// When creating a table that contains an internal LOB, the user can specify the chunking factor, which can be a multiple of Oracle
  /// Database blocks. This corresponds to the chunk size used by the LOB data layer when accessing and modifying the LOB value. Part
  /// of the chunk is used to store system-related information, and the rest stores the LOB value. This function returns the amount of
  /// space used in the LOB chunk to store the LOB value. Performance is improved if the application issues read or write requests using
  /// a multiple of this chunk size. For writes, there is an added benefit because LOB chunks are versioned and, if all writes are done
  /// on a chunk basis, no extra versioning is done or duplicated. Users could batch up the write until they have enough for a chunk
  /// instead of issuing several write calls for the same chunk.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - locp (IN/OUT):
  ///   The internal LOB for which to get the usable chunk size.
  /// - chunk_size (OUT):
  ///   * For LOBs with storage parameter `BASICFILE`, the amount of a chunk's space that is used to store the internal LOB value.
  ///     This is the amount that users should use when reading or writing the LOB value. If possible, users should start their
  ///     writes at chunk boundaries, such as the beginning of a chunk, and write a chunk at a time.
  ///   * The `chunk_size` parameter is returned in terms of bytes for `BLOB`s, `CLOB`s, and `NCLOB`s.
  ///   * For LOBs with storage parameter `SECUREFILE`, `chunk_size` is an advisory size and is provided for backward compatibility.
  fn OCILobGetChunkSize(svchp: *mut OCISvcCtx,
                        errhp: *mut OCIError,
                        // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                        locp: *mut c_void/*OCILobLocator*/,
                        chunk_size: *mut u32) -> c_int;
  /// Compares two LOB or `BFILE` locators for equality.
  ///
  /// Compares the given LOB or `BFILE` locators for equality. Two LOB or `BFILE` locators are equal if and only if they both refer to
  /// the same LOB or `BFILE` value.
  ///
  /// Two `NULL` locators are considered not equal by this function. 
  ///
  /// # Parameters
  /// - envhp (IN):
  ///   The OCI environment handle.
  /// - x (IN):
  ///   LOB locator to compare.
  /// - y (IN):
  ///   LOB locator to compare.
  /// - is_equal (OUT):
  ///   `TRUE`, if the LOB locators are equal; `FALSE` if they are not.
  fn OCILobIsEqual(envhp: *mut OCIEnv,
                   // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                   x: *const c_void/*OCILobLocator*/,
                   y: *const c_void/*OCILobLocator*/,
                   is_equal: *mut c_int) -> c_int;

//-------------------------------------------------------------------------------------------------
// Доступно только для BFILE
//-------------------------------------------------------------------------------------------------
  /// Sets the directory object and file name in the `BFILE` locator.
  ///
  /// It is an error to call this function for an internal LOB.
  ///
  /// # Parameters
  /// - envhp (IN/OUT):
  ///   OCI environment handle. Contains the UTF-16 setting.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - filepp (IN/OUT):
  ///   Pointer to the `BFILE` locator for which to set the directory object and file name.
  /// - dir_alias (IN):
  ///   Buffer that contains the directory object name (must be in the encoding specified by the charset parameter
  ///   of a previous call to `OCIEnvNlsCreate()`) to set in the `BFILE` locator.
  /// - d_length (IN):
  ///   Length (in bytes) of the input dir_alias parameter.
  /// - filename (IN):
  ///   Buffer that contains the file name (must be in the encoding specified by the charset parameter of a previous
  ///   call to `OCIEnvNlsCreate()`) to set in the `BFILE` locator.
  /// - f_length (IN):
  ///   Length (in bytes) of the input filename parameter.
  fn OCILobFileSetName(envhp: *mut OCIEnv,
                       errhp: *mut OCIError,
                       // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                       filepp: *mut *mut c_void/*OCILobLocator*/,
                       dir_alias: *const c_char,
                       d_length: u16,
                       filename: *const c_char,
                       f_length: u16) -> c_int;
  /// Tests to see if the `BFILE` exists on the server's operating system.
  ///
  /// Checks to see if the `BFILE` exists on the server's file system. It is an error to call this function for an internal LOB.
  ///
  /// # Parameters
  /// - svchp (IN):
  ///   The OCI service context handle.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - filep (IN):
  ///   Pointer to the `BFILE` locator that refers to the file.
  /// - flag (OUT):
  ///   Returns `TRUE` if the `BFILE` exists on the server; `FALSE` if it does not.
  fn OCILobFileExists(svchp: *mut OCISvcCtx,
                      errhp: *mut OCIError,
                      // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                      filep: *mut c_void/*OCILobLocator*/,
                      flag: *mut c_int) -> c_int;
//-------------------------------------------------------------------------------------------------
// Доступно только для CLOB/NCLOB
//-------------------------------------------------------------------------------------------------
  /// Gets the LOB locator's database character set ID of the LOB locator, if any.
  fn OCILobCharSetId(envhp: *mut OCIEnv,
                     errhp: *mut OCIError,
                     // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                     locp: *const c_void/*OCILobLocator*/,
                     csid: *mut u16) ->c_int;
}