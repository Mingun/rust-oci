//! Функции, описанные в разделе [Bind, Define, and Describe Functions][1] документации Oracle,
//! посвященном работе передачей параметров запросам и получению данных из результатов запросов.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/bind-define-describe-functions.htm#LNOCI153

use std::os::raw::{c_int, c_longlong, c_void, c_uchar, c_uint, c_ushort};
use super::{OCIBind, OCIDefine, OCIError, OCIStmt};

// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  pub fn OCIBindByName2(stmtp: *mut OCIStmt,
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
  /// - type (IN) [optional]:
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
                         type: *const OCIType,
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