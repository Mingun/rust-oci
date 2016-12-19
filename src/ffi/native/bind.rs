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
}