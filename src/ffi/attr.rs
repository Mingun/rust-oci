use std::ffi::CString;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use std::slice;
use num_integer::Integer;

use {DbResult, Result};

use ffi::{types, Handle};// Основные типобезопасные примитивы
use ffi::HandleType;// Типажи для безопасного моста к FFI

use ffi::native::OCIError;// FFI типы
use ffi::native::{OCIAttrGet, OCIAttrSet};// FFI функции

//-------------------------------------------------------------------------------------------------
/// Типаж, позволяющий получать и устанавливать атрибуты тем структурам, которые его реализуют.
pub trait AttrHolder<T> {
  /// Тип владельца атрибутов
  fn holder_type() -> u32;

  fn native(&self) -> *const T;

  fn native_mut(&mut self) -> *mut T {
    self.native() as *mut T
  }
  fn as_ref(&self) -> &T {
    unsafe { &*self.native() }
  }

  /// Получает значение указанного атрибута из объекта-владельца атрибутов
  unsafe fn get(&self, value: *mut c_void, size: &mut u32, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<()> {
    let res = OCIAttrGet(
      self.native() as *const c_void, Self::holder_type(),
      value, size, attrtype as u32,
      err.native_mut()
    );
    return err.check(res);
  }
  fn set(&mut self, value: *mut c_void, size: u32, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<()> {
    let res = unsafe {
      OCIAttrSet(
        self.native_mut() as *mut c_void, Self::holder_type(),
        value, size, attrtype as u32,
        err.native_mut()
      )
    };
    return err.check(res);
  }

//-------------------------------------------------------------------------------------------------
  fn get_<I: Integer>(&self, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<I> {
    let mut res = I::zero();
    let ptr = &mut res as *mut I;
    try!(unsafe { self.get(ptr as *mut c_void, &mut 0, attrtype, err) });

    Ok(res)
  }
  fn get_str(&self, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<String> {
    let mut len: u32 = 0;
    let mut str: *mut u8 = ptr::null_mut();
    let ptr = &mut str as *mut *mut u8;
    unsafe {
      try!(self.get(ptr as *mut c_void, &mut len, attrtype, err));
      //FIXME: Нужно избавиться от паники, должна возвращаться ошибка
      let cstr = CString::new(slice::from_raw_parts(str, len as usize)).expect("OCIAttrGet call returns string with embedded NUL byte");

      Ok(cstr.into_string().expect("OCIAttrGet call returns non UTF-8 string"))
    }
  }
//-------------------------------------------------------------------------------------------------
  fn set_<I: Integer>(&mut self, value: I, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<()> {
    let ptr = &value as *const I;
    self.set(ptr as *mut c_void, mem::size_of::<I>() as u32, attrtype, err)
  }
  /// Устанавливает строковый атрибут хендлу
  fn set_str(&mut self, value: &str, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<()> {
    self.set(value.as_ptr() as *mut c_void, value.len() as u32, attrtype, err)
  }
  /// Устанавливает хендл-атрибут хендлу
  fn set_handle<U: HandleType>(&mut self, value: &Handle<U>, attrtype: types::Attr, err: &Handle<OCIError>) -> DbResult<()> {
    self.set(value.native() as *mut c_void, 0, attrtype, err)
  }
}