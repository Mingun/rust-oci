//! Функции, описанные в разделе [Bind, Define, and Describe Functions][1] документации Oracle,
//! посвященном работе передачей параметров запросам и получению данных из результатов запросов.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI153

use std::os::raw::{c_int, c_void, c_uchar, c_uint, c_ushort};

use ffi::native::{OCIBind, OCISvcCtx, OCIDefine, OCIDescribe, OCIError, OCIStmt, OCIType};// FFI типы

// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  // Данное API доступно только с версии 12с, для которой пока нет Express версии, поэтому тестировать не на чем
  /*pub fn OCIBindByName2(stmtp: *mut OCIStmt,
                        defnpp: *mut *mut OCIBind,
                        errhp: *mut OCIError,
                        placeholder: *const c_uchar,
                        placeh_len: c_int,
                        valuep: *mut c_void,
                        value_sz: c_longlong,
                        dty: c_ushort,
                        indp: *mut c_void,
                        alenp: *mut c_uint,
                        rcodep: *mut c_ushort,
                        maxarr_len: c_uint,
                        curelep: *mut c_uint,
                        mode: c_uint) -> c_int;
  pub fn OCIBindByPos2(stmtp: *mut OCIStmt,
                       defnpp: *mut *mut OCIBind,
                       errhp: *mut OCIError,
                       position: c_uint,
                       valuep: *mut c_void,
                       value_sz: c_longlong,
                       dty: c_ushort,
                       indp: *mut c_void,
                       alenp: *mut c_uint,
                       rcodep: *mut c_ushort,
                       maxarr_len: c_uint,
                       curelep: *mut c_uint,
                       mode: c_uint) -> c_int;

  pub fn OCIDefineByPos2(stmtp: *mut OCIStmt,
                         defnpp: *mut *mut OCIDefine,
                         errhp: *mut OCIError,
                         position: c_uint,
                         valuep: *mut c_void,
                         value_sz: c_longlong,
                         dty: c_ushort,
                         indp: *mut c_void,
                         rlenp: *mut c_uint,
                         rcodep: *mut c_ushort,
                         mode: c_uint) -> c_int;*/

  pub fn OCIBindByName(stmtp: *mut OCIStmt,
                       defnpp: *mut *mut OCIBind,
                       errhp: *mut OCIError,
                       placeholder: *const c_uchar,
                       placeh_len: c_int,
                       valuep: *mut c_void,
                       value_sz: c_int,
                       dty: c_ushort,
                       indp: *mut c_void,
                       alenp: *mut c_ushort,
                       rcodep: *mut c_ushort,
                       maxarr_len: c_uint,
                       curelep: *mut c_uint,
                       mode: c_uint) -> c_int;
  pub fn OCIBindByPos(stmtp: *mut OCIStmt,
                      defnpp: *mut *mut OCIBind,
                      errhp: *mut OCIError,
                      position: c_uint,
                      valuep: *mut c_void,
                      value_sz: c_int,
                      dty: c_ushort,
                      indp: *mut c_void,
                      alenp: *mut c_ushort,
                      rcodep: *mut c_ushort,
                      maxarr_len: c_uint,
                      curelep: *mut c_uint,
                      mode: c_uint) -> c_int;

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
                        position: c_uint,
                        valuep: *mut c_void,
                        value_sz: c_int,
                        dty: c_ushort,
                        indp: *mut c_void,
                        rlenp: *mut c_ushort,
                        rcodep: *mut c_ushort,
                        mode: c_uint) -> c_int;

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
                         pvszsp: *mut c_uint,
                         indpp: *mut *mut c_void,
                         indszp: *mut c_uint) -> c_int;
  /// Describes existing schema and subschema objects.
  pub fn OCIDescribeAny(svchp: *mut OCISvcCtx,
                        errhp: *mut OCIError,
                        objptr: *mut c_void,
                        objptr_len: c_uint,
                        objptr_typ: c_uchar,
                        info_level: c_uchar,
                        objtyp: c_uchar,
                        dschp: *mut OCIDescribe) -> c_int;

  pub fn OCIStmtGetBindInfo(stmtp: *mut OCIStmt,
                            errhp: *mut OCIError,
                            size: c_uint,
                            startloc: c_uint,
                            found: *mut c_int,
                            bvnp: *mut *mut c_uchar,
                            bvnl: *mut c_uchar,
                            invp: *mut *mut c_uchar,
                            inpl: *mut c_uchar,
                            dupl: *mut c_uchar,
                            hndl: *mut *mut OCIBind) -> c_int;

}