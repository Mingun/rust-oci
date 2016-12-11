
use std::os::raw::{c_int, c_uchar, c_uint, c_ushort, c_void};
use super::{OCISvcCtx, OCIError, Handle, HandleType, Connection};
use super::types;

enum OCIStmt {}     impl HandleType for OCIStmt { const ID: types::Handle = types::Handle::Stmt; }
enum OCISnapshot {}

#[link(name = "oci")]
#[allow(dead_code)]
extern "C" {
  /// Associates an application request with a server.
  fn OCIStmtExecute(svchp: *mut OCISvcCtx,
                    stmtp: *mut OCIStmt,
                    errhp: *mut OCIError,
                    iters: c_uint,
                    rowoff: c_uint,
                    snap_in: *const OCISnapshot,
                    snap_out: *mut OCISnapshot,
                    mode: c_uint) -> c_int;
  /// Fetches a row from the (scrollable) result set. You are encouraged to use this fetch call instead of the deprecated call `OCIStmtFetch()`.
  fn OCIStmtFetch2(stmtp: *mut OCIStmt,
                   errhp: *mut OCIError,
                   nrows: c_uint,
                   orientation: c_ushort,
                   fetchOffset: c_int,
                   mode: c_uint) -> c_int;
  /// Returns the implicit results from an executed PL/SQL statement handle.
  fn OCIStmtGetNextResult(stmtp: *mut OCIStmt,
                          errhp: *mut OCIError,
                          result: *mut *mut c_void,
                          rtype: *mut c_uint,
                          mode: c_uint) -> c_int;
  /// Returns piece information for a piecewise operation.
  fn OCIStmtGetPieceInfo(stmtp: *const OCIStmt,
                         errhp: *mut OCIError,
                         hndlpp: *mut *mut c_void,
                         typep: *mut c_uint,
                         in_outp: *mut c_uchar,
                         iterp: *mut c_uint,
                         idxp: *mut c_uint,
                         piecep: *mut c_uchar) -> c_int;
  /// Prepares a SQL or PL/SQL statement for execution. The user has the option of using the statement cache, if it has been enabled.
  fn OCIStmtPrepare2(svchp: *mut OCISvcCtx,
                     stmthp: *mut *mut OCIStmt,
                     errhp: *mut OCIError,
                     stmttext: *const c_uchar,
                     stmt_len: c_uint,
                     key: *const c_uchar,
                     keylen: c_uint,
                     language: c_uint,
                     mode: c_uint) -> c_int;
  /// Releases the statement handle obtained by a call to `OCIStmtPrepare2()`.
  fn OCIStmtRelease(stmthp: *mut OCIStmt,
                    errhp: *mut OCIError,
                    key: *const c_uchar,
                    keylen: c_uint,
                    mode: c_uint) -> c_int;
  /// Sets piece information for a piecewise operation.
  fn OCIStmtSetPieceInfo(hndlp: *mut c_void,
                         htype: c_uint,
                         errhp: *mut OCIError,
                         bufp: *const c_void,
                         alenp: *mut c_uint,
                         piece: c_uchar,
                         indp: *const c_void, 
                         rcodep: *mut c_ushort) -> c_int;
}

pub struct Statement<'conn> {
  conn: Connection<'conn>,
  handle: Handle<OCIStmt>,
}
impl<'conn> Drop for Statement<'conn> {
  fn drop(&mut self) {
    unsafe { OCIStmtRelease(self.handle.native, 0 as *mut OCIError, 0 as *const c_uchar, 0, 0); }
  }
}