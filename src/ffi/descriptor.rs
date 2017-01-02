use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;
use std::slice;

use {Environment, Result};

use super::{check, Handle};
use super::base::AttrHolder;
use super::native::OCIError;// FFI типы
use super::native::{OCIDescriptorAlloc, OCIDescriptorFree};// FFI функции
use super::native::DescriptorType;// Типажи для безопасного моста к FFI
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый дескриптор ресурсов оракла
pub struct Descriptor<'d, T: 'd + DescriptorType> {
  native: *const T,
  phantom: PhantomData<&'d T>,
}
impl<'d, T: 'd + DescriptorType> Descriptor<'d, T> {
  pub fn new<'e>(env: &'e Environment) -> Result<Descriptor<'e, T>> {
    let mut desc = ptr::null_mut();
    let res = unsafe {
      OCIDescriptorAlloc(
        env.env.native() as *const c_void,
        &mut desc, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделенное под них место
      )
    };
    Self::from_ptr(res, desc as *const T, env.error())
  }
  pub fn from_ptr<'e>(res: c_int, native: *const T, err: &Handle<OCIError>) -> Result<Descriptor<'e, T>> {
    match res {
      0 => Ok(Descriptor { native: native, phantom: PhantomData }),
      e => Err(err.decode(e)),
    }
  }
  #[inline]
  pub fn address_mut(&mut self) -> *mut c_void {
    &mut self.native as *mut *const T as *mut c_void
  }
  #[inline]
  pub fn as_slice(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(
        &self.native as *const *const T as *const u8,
        mem::size_of::<*const T>()
      )
    }
  }
}
impl<'d, T: 'd + DescriptorType> Drop for Descriptor<'d, T> {
  fn drop(&mut self) {
    let res = unsafe { OCIDescriptorFree(self.native as *mut c_void, T::ID as c_uint) };
    //FIXME: Необходимо получать точную причину ошибки, а для этого нужна ссылка на OCIError.
    // Однако тащить ее в дескриптор нельзя, т.к. данная структура должна быть легкой
    check(res).expect("OCIDescriptorFree");
  }
}
impl<'d, T: 'd + DescriptorType> fmt::Debug for Descriptor<'d, T> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("Descriptor")
       .field(&T::ID)
       .field(&self.native)
       .finish()
  }
}
impl<'d, T: 'd + DescriptorType> AttrHolder<T> for Descriptor<'d, T> {
  fn holder_type() -> c_uint {
    T::ID as c_uint
  }

  fn native(&self) -> *const T {
    self.native
  }
}
