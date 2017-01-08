
use std::os::raw::{c_void, c_uint};
use std::mem::size_of;

use num_traits::{Signed, Unsigned};
use num_integer::Integer;

use {Connection, DbResult, Result};
use convert::FromDB;
use error::Error;
use types::Type;

use ffi::Handle;// Основные типобезопасные примитивы

use ffi::native::OCIError;// FFI типы
use ffi::native::num::OCINumberToInt;// FFI функции
use ffi::types::NumberFlag;

/// Родное представление числа Oracle-а.
#[derive(Debug)]
#[repr(C)]
pub struct OCINumber([u8; 22]);

impl OCINumber {
  /// Преобразует число из родного формата Oracle в беззнаковое число указанного размера.
  pub fn to_u<I: Integer + Unsigned>(&self, err: &Handle<OCIError>) -> Result<I> {
    self.to(err, NumberFlag::Unsigned).map_err(Into::into)
  }
  /// Преобразует число из родного формата Oracle в знаковое число указанного размера.
  pub fn to_i<I: Integer + Signed>(&self, err: &Handle<OCIError>) -> Result<I> {
    self.to(err, NumberFlag::Signed).map_err(Into::into)
  }
  fn to<I: Integer>(&self, err: &Handle<OCIError>, signed: NumberFlag) -> DbResult<I> {
    let mut result: I = I::zero();
    let res = unsafe {
      OCINumberToInt(
        err.native_mut(),
        self.0.as_ptr() as *const OCINumber,
        size_of::<I>() as c_uint,
        signed as c_uint,
        &mut result as *mut I as *mut c_void
      )
    };
    match res {
      0 => Ok(result),
      e => Err(err.decode(e)),
    }
  }
}
impl Default for OCINumber {
  fn default() -> Self {
    OCINumber([0; 22])
  }
}
impl<'conn> FromDB<'conn> for OCINumber {
  fn from_db(ty: Type, raw: &[u8], _: &Connection) -> Result<Self> {
    match ty {
      Type::VNU => {
        if raw.len() > 22 {
          return Err(Error::Overflow { extracted: raw.len(), capacity: 22 });
        }
        let mut r = OCINumber::default();
        r.0[0..raw.len()].clone_from_slice(raw);
        Ok(r)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}

macro_rules! simple_from {
  ($ty:ty, $($types:ident),+) => (
    impl<'conn> FromDB<'conn> for $ty {
      fn from_db(ty: Type, raw: &[u8], _: &Connection) -> Result<Self> {
        match ty {
          $(Type::$types)|+ => Ok(unsafe { *(raw.as_ptr() as *const $ty) }),
          t => Err(Error::Conversion(t)),
        }
      }
    }
  )
}
simple_from!(f32, FLT, BFLOAT);
simple_from!(f64, FLT, BDOUBLE);

// Чтобы оракл поместил данные в буфер в этих форматах, ему нужно при define-е указать соответствующую
// длину переменной, а сейчас там всегда указывается длина столбца. Таким образом, оракл всегда будет
// возвращать данные в VNU формате
simple_from!( i8, INT);
simple_from!(i16, INT);
simple_from!(i32, INT);
simple_from!(i64, INT);

simple_from!(u64, INT, UIN);