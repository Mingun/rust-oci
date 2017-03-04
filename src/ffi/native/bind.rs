//! Функции, описанные в разделе [Bind, Define, and Describe Functions][1] документации Oracle,
//! посвященном работе передачей параметров запросам и получению данных из результатов запросов.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI153

use std::fmt;
use std::mem;
use std::os::raw::{c_int, c_void};
use std::ptr;

use ffi::native::{OCIBind, OCISvcCtx, OCIDefine, OCIDescribe, OCIError, OCIStmt, OCIType};// FFI типы
use ffi::native::lob::LobPiece;
use ffi::types::{CallbackResult, OCIInd};


pub type OCICallbackInBind  = extern "C" fn(ictxp: *mut c_void,
                                            bindp: *mut OCIBind,
                                            iter: u32,
                                            index: u32,
                                            bufpp: *mut *mut c_void,
                                            alenp: *mut u32,
                                            piecep: *mut u8,
                                            indpp: *mut *mut c_void) -> i32;
pub type OCICallbackOutBind = extern "C" fn(octxp: *mut c_void,
                                            bindp: *mut OCIBind,
                                            iter: u32,
                                            index: u32,
                                            bufpp: *mut *mut c_void,
                                            alenpp: *mut *mut u32,
                                            piecep: *mut u8,
                                            indpp: *mut *mut c_void,
                                            rcodepp: *mut *mut u16) -> i32;

/// Функция, возвращающая значение связанной переменной.
///
/// # Параметры
/// - `handle`:
///   Хендл связанного параметра, уникально идентфицирующий параметр
/// - `store`:
///   Место, куда необходимо записать выходные данные
/// - `iter`:
///   A 0-based execute iteration value.
/// - `index`:
///   For PL/SQL, the index of the current array for an array bind. For SQL, the index is the row number
///   in the current iteration. It is 0-based, and must not be greater than the `curelep` parameter of
///   the bind call.
/// - `piece`:
///   A piece of the bind value. This can be one of the following values: `OCI_ONE_PIECE`, `OCI_FIRST_PIECE`,
///   `OCI_NEXT_PIECE`, and `OCI_LAST_PIECE`. For data types that do not support piecewise operations, you
///   must pass `OCI_ONE_PIECE` or an error is generated.
pub type InBindFn<'f> = FnMut(&mut OCIBind, &mut Vec<u8>, u32, u32, LobPiece) -> (bool, LobPiece, bool) + 'f;

/// Содержит информацию, позволяющую вызвать замыкание и сохранить полученные данные до тех пор,
/// пока они не будут переданы Oracle.
pub struct BindContext<'a> {
  /// Функция, предоставляющая данные для связанных переменных
  func: Box<InBindFn<'a>>,
  /// Место, где хранятся данные для связанной переменной, возвращенные замыканием, пока не будет
  /// вызван метод `execute`.
  store: Vec<u8>,
  /// Место для указания адреса в памяти, в котором хранится признак `NULL`-а в связанной переменной.
  /// По странной прихоти API требует указать адрес переменной, в которой хранится признак `NULL`-а,
  /// а не просто заполнить выходной параметр в функции обратного вызова.
  is_null: OCIInd,
}
impl<'a> BindContext<'a> {
  pub fn new<F>(f: F) -> Self
    where F: FnMut(&mut OCIBind, &mut Vec<u8>, u32, u32, LobPiece) -> (bool, LobPiece, bool) + 'a
  {
    BindContext {
      func: Box::new(f),
      store: Vec::new(),
      is_null: OCIInd::NotNull
    }
  }
}
impl<'a> fmt::Debug for BindContext<'a> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("BindContext")
       .field("func", &(&self.func as *const _))
       .field("store", &self.store)
       .field("is_null", &self.is_null)
       .finish()
  }
}
/// Функция для преобразования Rust-like замыкания в C-like функцию, требуемую в API Oracle.
pub extern "C" fn in_bind_adapter(ictxp: *mut c_void,
                                  bindp: *mut OCIBind,
                                  iter: u32,
                                  index: u32,
                                  bufpp: *mut *mut c_void,
                                  alenp: *mut u32,
                                  piecep: *mut u8,
                                  indpp: *mut *mut c_void) -> i32 {
  let ctx: &mut BindContext = unsafe { mem::transmute(ictxp) };
  let handle = unsafe { &mut *bindp };
  let s = &mut ctx.store;
  let piece: LobPiece = unsafe { mem::transmute(*piecep) };

  let (is_null, piece, res) = (ctx.func)(handle, s, iter, index, piece);
  ctx.is_null = if is_null { OCIInd::Null } else { OCIInd::NotNull };

  let (ptr, len) = match is_null {
    false => ( s.as_mut_ptr(), s.len()),
    true  => (ptr::null_mut(),       0),
  };

  unsafe {
    if !bufpp.is_null() { *bufpp = ptr as *mut c_void; }
    if !alenp.is_null() { *alenp = len as u32; }
    if !indpp.is_null() { *indpp = &ctx.is_null as *const _ as *const c_void as *mut c_void; }
    if !piecep.is_null(){ *piecep= piece as u8; }
  }

  (if res { CallbackResult::Done } else { CallbackResult::Continue }) as i32
}
// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  // Данное API доступно только с версии 12с, для которой пока нет Express версии, поэтому тестировать не на чем
  /*pub fn OCIBindByName2(stmtp: *mut OCIStmt,
                        defnpp: *mut *mut OCIBind,
                        errhp: *mut OCIError,
                        placeholder: *const u8,
                        placeh_len: i32,
                        valuep: *mut c_void,
                        value_sz: i64,
                        dty: u16,
                        indp: *mut c_void,
                        alenp: *mut u32,
                        rcodep: *mut u16,
                        maxarr_len: u32,
                        curelep: *mut u32,
                        mode: u32) -> c_int;
  pub fn OCIBindByPos2(stmtp: *mut OCIStmt,
                       defnpp: *mut *mut OCIBind,
                       errhp: *mut OCIError,
                       position: u32,
                       valuep: *mut c_void,
                       value_sz: i64,
                       dty: u16,
                       indp: *mut c_void,
                       alenp: *mut u32,
                       rcodep: *mut u16,
                       maxarr_len: u32,
                       curelep: *mut u32,
                       mode: u32) -> c_int;

  pub fn OCIDefineByPos2(stmtp: *mut OCIStmt,
                         defnpp: *mut *mut OCIDefine,
                         errhp: *mut OCIError,
                         position: u32,
                         valuep: *mut c_void,
                         value_sz: i64,
                         dty: u16,
                         indp: *mut c_void,
                         rlenp: *mut u32,
                         rcodep: *mut u16,
                         mode: u32) -> c_int;*/

  pub fn OCIBindByName(stmtp: *mut OCIStmt,
                       defnpp: *mut *mut OCIBind,
                       errhp: *mut OCIError,
                       placeholder: *const u8,
                       placeh_len: i32,
                       valuep: *mut c_void,
                       value_sz: i32,
                       dty: u16,
                       indp: *mut c_void,
                       alenp: *mut u16,
                       rcodep: *mut u16,
                       maxarr_len: u32,
                       curelep: *mut u32,
                       mode: u32) -> c_int;
  pub fn OCIBindByPos(stmtp: *mut OCIStmt,
                      defnpp: *mut *mut OCIBind,
                      errhp: *mut OCIError,
                      position: u32,
                      valuep: *mut c_void,
                      value_sz: i32,
                      dty: u16,
                      indp: *mut c_void,
                      alenp: *mut u16,
                      rcodep: *mut u16,
                      maxarr_len: u32,
                      curelep: *mut u32,
                      mode: u32) -> c_int;
  /// Registers user callbacks for dynamic data allocation.
  ///
  /// http://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI17142
  pub fn OCIBindDynamic(bindp: *mut OCIBind,
                        errhp: *mut OCIError,
                        ictxp: *mut c_void,
                        icbfp: Option<OCICallbackInBind>,
                        octxp: *mut c_void,
                        ocbfp: Option<OCICallbackOutBind>) -> c_int;

  /// Associates an item in a select list with the type and output data buffer.
  ///
  /// # Parameters
  /// - stmtp (IN/OUT):
  ///   A handle to the requested SQL query operation.
  /// - defnpp (IN/OUT):
  ///   A pointer to a pointer to a define handle. If this parameter is passed as `NULL`, this call implicitly allocates the define handle.
  ///   For a redefine, a non-`NULL` handle can be passed in this parameter. This handle is used to store the define information for this column.
  ///   Note:
  ///     You must keep track of this pointer. If a second call to OCIDefineByPos() is made for the same column position, there is no guarantee
  ///     that the same pointer will be returned.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - position (IN):
  ///   The position of this value in the select list. Positions are 1-based and are numbered from left to right. The value 0 selects `ROWID`s
  ///   (the globally unique identifier for a row in a table).
  /// - valuep (IN/OUT):
  ///   A pointer to a buffer or an array of buffers of the type specified in the `dty` parameter. A number of buffers can be specified when
  ///   results for more than one row are desired in a single fetch call.
  ///
  ///   For a `LOB`, the buffer pointer must be a pointer to a `LOB` locator of type `OCILobLocator`. Give the address of the pointer.
  ///
  ///   When mode is set to `OCI_IOV`, pass the base address of the `OCIIOV` struct.
  /// - value_sz (IN):
  ///   The size of each `valuep` buffer in bytes. If the data is stored internally in `VARCHAR2` format, the number of characters desired,
  ///   if different from the buffer size in bytes, can be specified as additional bytes by using `OCIAttrSet()`.
  ///
  ///   If the value of `value_sz > SB4MAXVAL`, an `ORA-24452` error will be issued, meaning that `values > SB4MAXVAL` are not supported in Release 12.1.
  ///
  ///   In a multibyte conversion environment, a truncation error is generated if the number of bytes specified is insufficient to handle the
  ///   number of characters needed.
  ///
  ///   If the `OCI_ATTR_CHARSET_ID` attribute is set to `OCI_UTF16ID` (replaces the deprecated `OCI_UCS2ID`, which is retained for backward
  ///   compatibility), all data passed to and received with the corresponding define call is assumed to be in UTF-16 encoding.
  ///
  ///   When mode is set to `OCI_IOV`, pass the size of the data value.
  /// - dty (IN):
  ///   The data type. Named data type (`SQLT_NTY`) and `REF` (`SQLT_REF`) are valid only if the environment has been initialized in object mode.
  ///   `SQLT_CHR` and `SQLT_LNG` can be specified for `CLOB` columns, and `SQLT_BIN` and `SQLT_LBI` can be specified for `BLOB` columns.
  /// - indp (IN):
  ///   Pointer to an indicator variable or array. For scalar data types, pointer to sb2 or an array of sb2s. Ignored for `SQLT_NTY` defines.
  ///   For `SQLT_NTY` defines, a pointer to a named data type indicator structure or an array of named data type indicator structures is
  ///   associated by a subsequent `OCIDefineObject()` call.
  /// - rlenp (IN/OUT):
  ///   Pointer to array of length of data fetched in bytes.
  /// - rcodep (OUT):
  ///   Pointer to array of column-level return codes.
  /// - mode (IN):
  ///   The valid modes are:
  ///   * `OCI_DEFAULT` - This is the default mode.
  ///   * `OCI_DEFINE_SOFT` - Soft define mode. This mode increases the performance of the call. If this is the first define, or some input parameter
  ///     such as dty or value_sz is changed from the previous define, this mode is ignored. Unexpected behavior results if an invalid define handle
  ///     is passed. An error is returned if the statement is not executed.
  ///   * `OCI_DYNAMIC_FETCH` - For applications requiring dynamically allocated data at the time of fetch, this mode must be used. You can define
  ///     a callback using the `OCIDefineDynamic()` call. The `value_sz` parameter defines the maximum size of the data that is to be provided at run
  ///     time. When the client library needs a buffer to return the fetched data, the callback is invoked to provide a runtime buffer into which
  ///     a piece or all the data is returned.
  pub fn OCIDefineByPos(stmtp: *mut OCIStmt,
                        defnpp: *mut *mut OCIDefine,
                        errhp: *mut OCIError,
                        position: u32,
                        valuep: *mut c_void,
                        value_sz: i32,
                        dty: u16,
                        indp: *mut c_void,
                        rlenp: *mut u16,
                        rcodep: *mut u16,
                        mode: u32) -> c_int;

  /// Sets up additional attributes necessary for a named data type or `REF` define.
  ///
  /// # Comments
  /// This function follows a call to `OCIDefineByPos()` to set initial define information. This call
  /// sets up additional attributes necessary for a named data type define. An error is returned if
  /// this function is called when the OCI environment has been initialized in non-object mode.
  ///
  /// This call takes as a parameter a type descriptor object (TDO) of data type `OCIType` for the
  /// named data type being defined. The TDO can be retrieved with a call to `OCIDescribeAny()`.
  ///
  /// # Parameters
  /// - defnp (IN/OUT):
  ///   A define handle previously allocated in a call to `OCIDefineByPos()`.
  /// - errhp (IN/OUT):
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - tdo (IN) [optional]:
  ///   Points to the type descriptor object (TDO) that describes the type of the program variable.
  ///   This parameter is optional for variables of type `SQLT_REF`, and may be passed as `NULL` if
  ///   it is not being used.
  /// - pgvpp (IN/OUT):
  ///   Points to a pointer to a program variable buffer. For an array, `pgvpp` points to an array of
  ///   pointers. Memory for the fetched named data type instances is dynamically allocated in the
  ///   object cache. At the end of the fetch when all the values have been received, `pgvpp` points
  ///   to the pointers to these newly allocated named data type instances. The application must call
  ///   `OCIObjectFree()` to deallocate the named data type instances when they are no longer needed.
  ///   If the application wants the buffer to be implicitly allocated in the cache, `*pgvpp` should
  ///   be passed in as `NULL`.
  /// - pvszsp (IN/OUT):
  ///   Points to the size of the program variable. For an array, it is an array of `ub4`.
  /// - indpp (IN/OUT):
  ///   Points to a pointer to the program variable buffer containing the parallel indicator structure.
  ///   For an array, points to an array of pointers. Memory is allocated to store the indicator structures
  ///   in the object cache. At the end of the fetch when all values have been received, `indpp` points
  ///   to the pointers to these newly allocated indicator structures.
  /// - indszp (IN/OUT):
  ///   Points to the sizes of the indicator structure program variable. For an array, it is an array of `ub4`s.
  pub fn OCIDefineObject(defnp: *mut OCIDefine,
                         errhp: *mut OCIError,
                         tdo: *const OCIType,
                         pgvpp: *mut *mut c_void,
                         pvszsp: *mut u32,
                         indpp: *mut *mut c_void,
                         indszp: *mut u32) -> c_int;
  /// Describes existing schema and subschema objects.
  pub fn OCIDescribeAny(svchp: *mut OCISvcCtx,
                        errhp: *mut OCIError,
                        objptr: *mut c_void,
                        objptr_len: u32,
                        objptr_typ: u8,
                        info_level: u8,
                        objtyp: u8,
                        dschp: *mut OCIDescribe) -> c_int;

  pub fn OCIStmtGetBindInfo(stmtp: *mut OCIStmt,
                            errhp: *mut OCIError,
                            size: u32,
                            startloc: u32,
                            found: *mut i32,
                            bvnp: *mut *mut u8,
                            bvnl: *mut u8,
                            invp: *mut *mut u8,
                            inpl: *mut u8,
                            dupl: *mut u8,
                            hndl: *mut *mut OCIBind) -> c_int;

}