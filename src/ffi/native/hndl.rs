//! Функции, описанные в разделе [Handle and Descriptor Functions][1] документации Oracle.
//!
//! [1]: http://docs.oracle.com/database/121/LNOCI/oci16rel002.htm#LNOCI152

use std::os::raw::{c_int, c_void, c_uint};
use super::OCIError;

// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Returns a pointer to an allocated and initialized handle.
  ///
  /// # Parameters
  /// - parenth:
  ///   An environment handle.
  /// - hndlpp:
  ///   Returns a handle.
  /// - htype:
  ///   Specifies the type of handle to be allocated. The allowed handles are described in Table 2-1.
  /// - xtramem_sz:
  ///   Specifies an amount of user memory to be allocated.
  /// - usrmempp:
  ///   Returns a pointer to the user memory of size xtramem_sz allocated by the call for the user.
  pub fn OCIHandleAlloc(parenth: *const c_void,
                        hndlpp: *mut *mut c_void, // результат
                        htype: c_uint,
                        xtramem_sz: c_uint,
                        usrmempp:  *mut *mut c_void // результат
                       ) -> c_int;
  /// This call explicitly deallocates a handle.
  ///
  /// # Comments
  /// This call frees up storage associated with a handle, corresponding to the type specified in the type parameter.
  ///
  /// This call returns either `OCI_SUCCESS`, `OCI_INVALID_HANDLE`, or `OCI_ERROR`.
  ///
  /// All handles may be explicitly deallocated. The OCI deallocates a child handle if the parent is deallocated.
  ///
  /// When a statement handle is freed, the cursor associated with the statement handle is closed, but the actual
  /// cursor closing may be deferred to the next round-trip to the server. If the application must close the cursor
  /// immediately, you can make a server round-trip call, such as `OCIServerVersion()` or `OCIPing()`, after the
  /// `OCIHandleFree()` call.
  ///
  /// # Parameters
  /// - hndlp:
  ///   A handle allocated by `OCIHandleAlloc()`.
  /// - htype:
  ///   Specifies the type of storage to be freed. The handles are described in Table 2-1.
  pub fn OCIHandleFree(hndlp: *mut c_void,
                       htype: c_uint) -> c_int;

  /// Allocates storage to hold descriptors or LOB locators.
  ///
  /// # Comments
  /// Returns a pointer to an allocated and initialized descriptor, corresponding to the type specified
  /// in `dtype`. A non-`NULL` descriptor or LOB locator is returned on success. No diagnostics are
  /// available on error.
  ///
  /// This call returns `OCI_SUCCESS` if successful, or `OCI_INVALID_HANDLE` if an out-of-memory error occurs.
  ///
  /// # Parameters
  /// - parenth:
  ///   An environment handle.
  /// - descpp:
  ///   Returns a descriptor or LOB locator of the desired type.
  /// - dtype:
  ///   Specifies the type of descriptor or LOB locator to be allocated.
  pub fn OCIDescriptorAlloc(parenth: *const c_void,
                            descpp: *mut *mut c_void, 
                            dtype: c_uint,
                            xtramem_sz: c_uint,
                            usrmempp: *mut *mut c_void) -> c_int;
  /// Deallocates a previously allocated descriptor.
  ///
  /// # Comments
  /// This call frees storage associated with a descriptor. Returns `OCI_SUCCESS` or `OCI_INVALID_HANDLE`.
  /// All descriptors can be explicitly deallocated; however, OCI deallocates a descriptor if the environment
  /// handle is deallocated.
  ///
  /// If you perform LOB operations, you must always call `OCILobFreeTemporary()` before calling `OCIDescriptorFree()`
  /// to free the contents of the temporary LOB. See About Freeing Temporary LOBs for more information.
  ///
  /// # Parameters
  /// - descp:
  ///   An allocated descriptor.
  /// - type:
  ///   Specifies the type of storage to be freed.
  pub fn OCIDescriptorFree(descp: *mut c_void,
                           dtype: c_uint) -> c_int;

  /// Returns a descriptor of a parameter specified by position in the describe handle or statement handle.
  ///
  /// # Comments
  /// This call returns a descriptor of a parameter specified by position in the describe handle or statement handle.
  /// Parameter descriptors are always allocated internally by the OCI library. They can be freed using `OCIDescriptorFree()`.
  /// For example, if you fetch the same column metadata for every execution of a statement, then the program leaks memory
  /// unless you explicitly free the parameter descriptor between each call to `OCIParamGet()`.
  ///
  /// # Parameters
  /// - hndlp:
  ///   A statement handle or describe handle. The `OCIParamGet()` function returns a parameter descriptor for this handle.
  /// - htype:
  ///   The type of the handle passed in the hndlp parameter.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - parmdpp:
  ///   A descriptor of the parameter at the position given in the `pos` parameter, of handle type `OCI_DTYPE_PARAM`.
  /// - pos:
  ///   Position number in the statement handle or describe handle. A parameter descriptor is returned for this position.
  pub fn OCIParamGet(hndlp: *const c_void,
                     htype: c_uint,
                     errhp: *mut OCIError,
                     parmdpp:  *mut *mut c_void,
                     pos: c_uint) -> c_int;
  /// Sets a complex object retrieval (COR) descriptor into a COR handle.
  ///
  /// # Comments
  /// The COR handle must have been previously allocated using `OCIHandleAlloc()`, and the descriptor must have
  /// been previously allocated using `OCIDescriptorAlloc()`. Attributes of the descriptor are set using `OCIAttrSet()`.
  ///
  /// # Parameters
  /// - hndlp:
  ///   Handle pointer.
  /// - htype:
  ///   Handle type.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - dscp:
  ///   Complex object retrieval descriptor pointer.
  /// - dtyp:
  ///   Descriptor type. The descriptor type for a COR descriptor is `OCI_DTYPE_COMPLEXOBJECTCOMP`.
  /// - pos:
  ///   Position number.
  pub fn OCIParamSet(hndlp: *mut c_void,
                     htype: c_uint,
                     errhp: *mut OCIError,
                     dscp: *const c_void,
                     dtyp: c_uint,
                     pos: c_uint);

  /// Gets the value of an attribute of a handle.
  pub fn OCIAttrGet(trgthndlp: *const c_void,
                    trghndltyp: c_uint,
                    attributep: *mut c_void,
                    sizep: *mut c_uint,
                    attrtype: c_uint,
                    errhp: *mut OCIError) -> c_int;
  /// Sets the value of an attribute of a handle or a descriptor.
  pub fn OCIAttrSet(trgthndlp: *mut c_void,
                    trghndltyp: c_uint,
                    attributep: *mut c_void,
                    size: c_uint,
                    attrtype: c_uint,
                    errhp: *mut OCIError) -> c_int;
}