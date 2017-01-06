//! Функции, описанные в разделе [LOB Functions][1] документации Oracle,
//! посвященном работе с большими объектами.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/lob-functions.htm#LNOCI162

use std::os::raw::{c_int, c_void, c_uchar, c_uint, c_ulonglong, c_ushort};

use ffi::types;
use ffi::native::{OCIEnv, OCIError, OCILobLocator, OCISvcCtx};// FFI типы

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
                      dst_locp: *mut OCILobLocator,
                      src_locp: *mut OCILobLocator) -> c_int;

  /// Reads LOB data for multiple locators in one round-trip.
  /// This function can be used for LOBs of size greater than or less than 4 GB.
  pub fn OCILobArrayRead(svchp: *mut OCISvcCtx,
                         errhp: *mut OCIError,
                         array_iter: *mut c_uint,
                         locp_arr: *mut *mut OCILobLocator,
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
                          locp_arr: *mut *mut OCILobLocator,
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
                      src_locp: *const OCILobLocator,
                      dst_locpp: *mut *mut OCILobLocator) -> c_int;
}