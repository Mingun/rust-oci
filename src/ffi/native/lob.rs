//! Функции, описанные в разделе [LOB Functions][1] документации Oracle,
//! посвященном работе с большими объектами.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/lob-functions.htm#LNOCI162

use std::io;
use std::os::raw::{c_int, c_void, c_uchar, c_uint, c_ulonglong, c_ushort};
use std::ptr;

use {Connection, DbResult};

use ffi::Descriptor;// Основные типобезопасные примитивы
use ffi::DescriptorType;// Типажи для безопасного моста к FFI

use ffi::attr::AttrHolder;
use ffi::types;
use ffi::native::{OCIEnv, OCIError, OCISvcCtx};// FFI типы

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
pub enum Charset {
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
#[derive(Debug)]
pub struct LobImpl<'conn, L: 'conn + OCILobLocator> {
  conn: &'conn Connection<'conn>,
  locator: Descriptor<'conn, L>,
}
impl<'conn, L: 'conn + OCILobLocator> LobImpl<'conn, L> {
  pub fn new(conn: &'conn Connection) -> DbResult<Self> {
    Ok(LobImpl { conn: conn, locator: try!(conn.server.new_descriptor()) })
  }
  pub fn len(&self) -> DbResult<u64> {
    let mut len = 0;
    let res = unsafe {
      OCILobGetLength2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator.native() as *const c_void as *mut c_void,
        &mut len
      )
    };
    try!(self.conn.error().check(res));

    Ok(len)
  }
  pub fn trim(&mut self, len: u64) -> DbResult<()> {
    let res = unsafe {
      OCILobTrim2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator.native_mut() as *mut c_void,
        len
      )
    };
    self.conn.error().check(res)
  }
  pub fn read(&mut self, offset: u64, piece: LobPiece, charset: u16, buf: &mut [u8]) -> DbResult<usize> {
    // Количество того, сколько читать и сколько было реально прочитано
    let mut readed = buf.len() as u64;
    let res = unsafe {
      OCILobRead2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator.native_mut() as *mut c_void,
        // Всегда задаем чтение в байтах, даже для [N]CLOB-ов
        &mut readed, ptr::null_mut(),
        offset,
        buf.as_mut_ptr() as *mut c_void, buf.len() as u64,
        piece as u8,
        // Функцию обратного вызова не используем
        ptr::null_mut(), None,
        charset, Charset::Implicit as u8
      )
    };
    try!(self.conn.error().check(res));

    // Не может быть прочитано больше, чем было запрошено, а то, что было запрошено,
    // не превышает usize.
    Ok(readed as usize)
  }
  pub fn write(&mut self, offset: u64, piece: LobPiece, charset: u16, buf: &[u8]) -> DbResult<usize> {
    // Количество того, сколько писать и сколько было реально записано
    let mut writed = buf.len() as u64;
    let res = unsafe {
      OCILobWrite2(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator.native_mut() as *mut c_void,
        // Всегда задаем запись в байтах, даже для [N]CLOB-ов
        &mut writed, ptr::null_mut(),
        offset,// имеет значение только при первом вызове, при последующих игнорируется
        buf.as_ptr() as *mut c_void, buf.len() as u64,
        piece as u8,
        // Функцию обратного вызова не используем
        ptr::null_mut(), None,
        // Данные параметры игнорируются для BLOB-ов.
        charset, Charset::Implicit as u8
      )
    };
    try!(self.conn.error().check(res));

    // Не может быть прочитано больше, чем было запрошено, а то, что было запрошено,
    // не превышает usize.
    Ok(writed as usize)
  }
  pub fn open(&mut self, mode: LobOpenMode) -> DbResult<()> {
    let res = unsafe {
      OCILobOpen(
        self.conn.context.native_mut(),
        self.conn.error().native_mut(),
        self.locator.native_mut() as *mut c_void,
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
        self.locator.native_mut() as *mut c_void
      )
    };
    self.conn.error().check(res)
  }

  pub fn new_reader(&'conn mut self, charset: u16) -> DbResult<LobReader<'conn, L>> {
    try!(self.open(LobOpenMode::ReadOnly));
    Ok(LobReader { lob: self, charset: charset })
  }
  pub fn new_writer(&'conn mut self, charset: u16) -> DbResult<LobWriter<'conn, L>> {
    try!(self.open(LobOpenMode::WriteOnly));
    Ok(LobWriter { lob: self, charset: charset })
  }
}
#[derive(Debug)]
pub struct LobReader<'lob, L: 'lob + OCILobLocator> {
  lob: &'lob mut LobImpl<'lob, L>,
  charset: u16,
}
impl<'lob, L: 'lob + OCILobLocator> io::Read for LobReader<'lob, L> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.lob.read(1, LobPiece::One, self.charset, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }
}
impl<'lob, L: 'lob + OCILobLocator> Drop for LobReader<'lob, L> {
  fn drop(&mut self) {
    self.lob.close().expect("Error when close LOB");
  }
}

#[derive(Debug)]
pub struct LobWriter<'lob, L: 'lob + OCILobLocator> {
  lob: &'lob mut LobImpl<'lob, L>,
  charset: u16,
}
impl<'lob, L: 'lob + OCILobLocator> io::Write for LobWriter<'lob, L> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.lob.write(1, LobPiece::One, self.charset, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
impl<'lob, L: 'lob + OCILobLocator> Drop for LobWriter<'lob, L> {
  fn drop(&mut self) {
    self.lob.close().expect("Error when close LOB");
  }
}

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
                          parent: c_ushort,
                          duration: *mut c_ushort// результат
                          ) -> c_int;
  /// Terminates a user duration for a temporary LOB.
  pub fn OCIDurationEnd(env: *mut OCIEnv,
                        err: *mut OCIError,
                        svc: *const OCISvcCtx,
                        duration: c_ushort) -> c_int;

  /// Appends a LOB value at the end of another LOB as specified.
  pub fn OCILobAppend(svchp: *mut OCISvcCtx,
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
  pub fn OCILobAssign(envhp: *mut OCIEnv, 
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
}