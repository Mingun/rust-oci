//! Функции, описанные в разделе [Statement Functions][1] документации Oracle,
//! посвященном подготовке и исполнению запросов (не включая получение данных `SELECT`-запросов).
//!
//! [1]: http://docs.oracle.com/database/121/LNOCI/oci17msc001.htm#LNOCI161

use std::os::raw::{c_int, c_uchar, c_uint, c_ushort, c_void};
use super::{OCIError, OCISvcCtx, OCISnapshot, OCIStmt};


// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Associates an application request with a server.
  pub fn OCIStmtExecute(svchp: *mut OCISvcCtx,
                        stmtp: *mut OCIStmt,
                        errhp: *mut OCIError,
                        iters: c_uint,
                        rowoff: c_uint,
                        snap_in: *const OCISnapshot,
                        snap_out: *mut OCISnapshot,
                        mode: c_uint) -> c_int;
  /// Fetches a row from the (scrollable) result set. You are encouraged to use this fetch call instead of the deprecated call `OCIStmtFetch()`.
  pub fn OCIStmtFetch2(stmtp: *mut OCIStmt,
                       errhp: *mut OCIError,
                       nrows: c_uint,
                       orientation: c_ushort,
                       fetchOffset: c_int,
                       mode: c_uint) -> c_int;
  /// Returns the implicit results from an executed PL/SQL statement handle.
  pub fn OCIStmtGetNextResult(stmtp: *mut OCIStmt,
                              errhp: *mut OCIError,
                              result: *mut *mut c_void,
                              rtype: *mut c_uint,
                              mode: c_uint) -> c_int;
  /// Returns piece information for a piecewise operation.
  pub fn OCIStmtGetPieceInfo(stmtp: *const OCIStmt,
                             errhp: *mut OCIError,
                             hndlpp: *mut *mut c_void,
                             typep: *mut c_uint,
                             in_outp: *mut c_uchar,
                             iterp: *mut c_uint,
                             idxp: *mut c_uint,
                             piecep: *mut c_uchar) -> c_int;
  /// Prepares a SQL or PL/SQL statement for execution. The user has the option of using the statement cache, if it has been enabled.
  pub fn OCIStmtPrepare2(svchp: *mut OCISvcCtx,
                         stmthp: *mut *mut OCIStmt,
                         errhp: *mut OCIError,
                         stmttext: *const c_uchar,
                         stmt_len: c_uint,
                         key: *const c_uchar,
                         keylen: c_uint,
                         language: c_uint,
                         mode: c_uint) -> c_int;
  /// Releases the statement handle obtained by a call to `OCIStmtPrepare2()`.
  pub fn OCIStmtRelease(stmthp: *mut OCIStmt,
                        errhp: *mut OCIError,
                        key: *const c_uchar,
                        keylen: c_uint,
                        mode: c_uint) -> c_int;
  /// Sets piece information for a piecewise operation.
  pub fn OCIStmtSetPieceInfo(hndlp: *mut c_void,
                             htype: c_uint,
                             errhp: *mut OCIError,
                             bufp: *const c_void,
                             alenp: *mut c_uint,
                             piece: c_uchar,
                             indp: *const c_void, 
                             rcodep: *mut c_ushort) -> c_int;
}