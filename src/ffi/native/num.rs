//! Функции, описанные в разделе [OCI NUMBER Functions][1] документации Oracle,
//! посвященном работе с числами.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/oci-NUMBER-functions.htm

use std::os::raw::{c_int, c_void, c_uchar, c_uint};

use types::OCINumber;

use ffi::native::OCIError;// FFI типы

// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Converts an Oracle `NUMBER` type to integer.
  ///
  /// # Comments
  /// This is a native type conversion function. It converts the given Oracle `NUMBER` into an integer
  /// of the form `xbn`, such as `ub2`, `ub4`, or `sb2`.
  ///
  /// # Returns
  /// This function returns an error if `number` or `rsl` is `NULL`, if `number` is too big (overflow)
  /// or too small (underflow), or if an invalid sign flag value is passed in `rsl_flag`.
  ///
  /// # Parameters
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - number (IN):
  ///   Oracle `NUMBER` to convert.
  /// - rsl_length (IN):
  ///   Size of the desired result.
  /// - rsl_flag (IN):
  ///   Flag that designates the sign of the output, as follows:
  ///   * `OCI_NUMBER_UNSIGNED` - Unsigned values
  ///   * `OCI_NUMBER_SIGNED` - Signed values
  /// - rsl (OUT):
  ///   Pointer to space for the result.
  pub fn OCINumberToInt(err: *mut OCIError,
                        number: *const OCINumber,
                        rsl_length: c_uint,
                        rsl_flag: c_uint,
                        rsl: *mut c_void) -> c_int;
  /// Converts an Oracle `NUMBER` type to a real type.
  ///
  /// # Comments
  /// This is a native type conversion function. It converts an Oracle `NUMBER` into a system-native real type.
  /// This function only converts `NUMBER`s up to `LDBL_DIG`, `DBL_DIG`, or `FLT_DIG` digits of precision and
  /// removes trailing zeros. These constants are defined in `float.h`.
  ///
  /// You must pass a valid OCINumber to this function. Otherwise, the result is undefined.
  ///
  /// # Parameters
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - number (IN):
  ///   Oracle `NUMBER` to convert.
  /// - rsl_length (IN):
  ///   The size of the desired result, which equals `sizeof({ float | double | long double})`.
  /// - rsl (OUT):
  ///   Pointer to space for the result.
  pub fn OCINumberToReal(err: *mut OCIError,
                         number: *const OCINumber,
                         rsl_length: c_uint,
                         rsl: *mut c_void) -> c_int;
  /// Converts an array of `NUMBER` to an array of real type.
  pub fn OCINumberToRealArray(err: *mut OCIError,
                              number: *const *const OCINumber,
                              elems: c_uint,
                              rsl_length: c_uint,
                              rsl: *mut c_void) -> c_int;
  /// Converts an Oracle `NUMBER` to a character string according to a specified format.
  pub fn OCINumberToText(err: *mut OCIError,
                         number: *const OCINumber,
                         fmt: *const c_uchar,
                         fmt_length: c_uint,
                         nls_params: *const c_uchar,
                         nls_p_length: c_uint,
                         buf_size: *mut c_uint,
                         buf: *mut c_uchar) -> c_int;
}
