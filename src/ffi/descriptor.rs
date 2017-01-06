use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;
use std::slice;

use {Environment, Result};

use ffi::{check, Handle};// Основные типобезопасные примитивы
use ffi::DescriptorType;// Типажи для безопасного моста к FFI

use ffi::attr::AttrHolder;
use ffi::native::OCIError;// FFI типы
use ffi::native::{OCIDescriptorAlloc, OCIDescriptorFree};// FFI функции
use ffi::types;

fn close<T>(native: *const T, id: types::Descriptor) {
  let res = unsafe { OCIDescriptorFree(native as *mut c_void, id as c_uint) };
  //FIXME: Необходимо получать точную причину ошибки, а для этого нужна ссылка на OCIError.
  // Однако тащить ее в дескриптор нельзя, т.к. данная структура должна быть легкой
  check(res).expect("OCIDescriptorFree");
}

//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый дескриптор ресурсов оракла
pub struct Descriptor<'d, T: 'd + DescriptorType> {
  native: *const T,
  phantom: PhantomData<&'d T>,
}
impl<'d, T: 'd + DescriptorType> Descriptor<'d, T> {
  pub fn new(env: &'d Environment) -> Result<Self> {
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
  pub fn from_ptr(res: c_int, native: *const T, err: &Handle<OCIError>) -> Result<Self> {
    match res {
      0 => Ok(Descriptor { native: native, phantom: PhantomData }),
      e => Err(err.decode(e)),
    }
  }
}
impl<'d, T: 'd + DescriptorType> Drop for Descriptor<'d, T> {
  fn drop(&mut self) {
    close(self.native, T::ID);
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

/// Нетипизированный автоматически закрываемый дескриптор оракла.
pub struct GenericDescriptor<'d> {
  native: *const c_void,
  id: types::Descriptor,
  phantom: PhantomData<&'d c_void>,
}
impl<'d> GenericDescriptor<'d> {
  /// Получает указатель на начало памяти, в которой лежит указатель на данные дескриптора
  #[inline]
  pub fn address_mut(&mut self) -> *mut c_void {
    &mut self.native as *mut *const c_void as *mut c_void
  }
  /// Получает срез, представляющий кусок памяти, в котором лежит указатель на данные дескриптора
  #[inline]
  pub fn as_slice(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(
        &self.native as *const *const c_void as *const u8,
        mem::size_of::<*const c_void>()
      )
    }
  }
}
impl<'d, T: DescriptorType> From<Descriptor<'d, T>> for GenericDescriptor<'d> {
  fn from(d: Descriptor<'d, T>) -> Self {
    let res = GenericDescriptor { native: d.native as *const c_void, id: T::ID, phantom: PhantomData };
    // Дескриптор уходит в небытие, чтобы он не закрыл ресурс, забываем его
    mem::forget(d);
    res
  }
}
impl<'d> Drop for GenericDescriptor<'d> {
  fn drop(&mut self) {
    close(self.native, self.id);
  }
}
impl<'d> fmt::Debug for GenericDescriptor<'d> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("GenericDescriptor")
       .field(&self.id)
       .field(&self.native)
       .finish()
  }
}