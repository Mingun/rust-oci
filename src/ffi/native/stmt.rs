//! Функции, описанные в разделе [Statement Functions][1] документации Oracle,
//! посвященном подготовке и исполнению запросов (не включая получение данных `SELECT`-запросов).
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/statement-functions.htm#LNOCI161

use std::os::raw::{c_int, c_void};

use ffi::native::{OCIError, OCISvcCtx, OCISnapshot, OCIStmt};// FFI типы


// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Associates an application request with a server.
  pub fn OCIStmtExecute(svchp: *mut OCISvcCtx,
                        stmtp: *mut OCIStmt,
                        errhp: *mut OCIError,
                        iters: u32,
                        rowoff: u32,
                        snap_in: *const OCISnapshot,
                        snap_out: *mut OCISnapshot,
                        mode: u32) -> c_int;
  /// Fetches a row from the (scrollable) result set. You are encouraged to use this fetch call instead of the deprecated call `OCIStmtFetch()`.
  pub fn OCIStmtFetch2(stmtp: *mut OCIStmt,
                       errhp: *mut OCIError,
                       nrows: u32,
                       orientation: u16,
                       fetchOffset: i32,
                       mode: u32) -> c_int;
  /// Returns the implicit results from an executed PL/SQL statement handle.
  pub fn OCIStmtGetNextResult(stmtp: *mut OCIStmt,
                              errhp: *mut OCIError,
                              result: *mut *mut c_void,
                              rtype: *mut u32,
                              mode: u32) -> c_int;
  /// Returns piece information for a piecewise operation.
  pub fn OCIStmtGetPieceInfo(stmtp: *const OCIStmt,
                             errhp: *mut OCIError,
                             hndlpp: *mut *mut c_void,
                             typep: *mut u32,
                             in_outp: *mut u8,
                             iterp: *mut u32,
                             idxp: *mut u32,
                             piecep: *mut u8) -> c_int;
  /// Prepares a SQL or PL/SQL statement for execution. The user has the option of using the statement cache, if it has been enabled.
  pub fn OCIStmtPrepare2(svchp: *mut OCISvcCtx,
                         stmthp: *mut *mut OCIStmt,
                         errhp: *mut OCIError,
                         stmttext: *const u8,
                         stmt_len: u32,
                         key: *const u8,
                         keylen: u32,
                         language: u32,
                         mode: u32) -> c_int;
  /// Releases the statement handle obtained by a call to `OCIStmtPrepare2()`.
  pub fn OCIStmtRelease(stmthp: *mut OCIStmt,
                        errhp: *mut OCIError,
                        key: *const u8,
                        keylen: u32,
                        mode: u32) -> c_int;
  /// Sets piece information for a piecewise operation.
  pub fn OCIStmtSetPieceInfo(hndlp: *mut c_void,
                             htype: u32,
                             errhp: *mut OCIError,
                             bufp: *const c_void,
                             alenp: *mut u32,
                             piece: u8,
                             indp: *const c_void, 
                             rcodep: *mut u16) -> c_int;
}