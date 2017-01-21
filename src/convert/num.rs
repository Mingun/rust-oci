
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
    println!("ty: {:?} [{:?}]{:?}", ty, raw.len(), raw);
    match ty {
      Type::NUM => {
        let l = raw.len();
        if l > 21 {
          return Err(Error::Overflow { extracted: l, capacity: 21 });
        }
        // В соответствии с примером в http://docs.oracle.com/database/122/LNOCI/object-relational-data-types-in-oci.htm#LNOCI16848
        // можно так копаться во внутренней структуре числа.
        let mut r = OCINumber::default();
        r.0[0] = l as u8;
        r.0[1..(1+l)].clone_from_slice(raw);
        Ok(r)
      }
      Type::VNU => {
        let l = raw.len();
        if l > 22 {
          return Err(Error::Overflow { extracted: l, capacity: 22 });
        }
        let mut r = OCINumber::default();
        r.0[0..l].clone_from_slice(raw);
        Ok(r)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}

macro_rules! num_from {
  ($ty:ty, $sign:expr, $($types:ident),+) => (
    impl<'conn> FromDB<'conn> for $ty {
      fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
        match ty {
          $(Type::$types)|+ => Ok(unsafe { *(raw.as_ptr() as *const $ty) }),
          t => {
            let num = try!(OCINumber::from_db(t, raw, conn));
            num.to::<$ty>(conn.error(), $sign).map_err(Into::into)
          },
        }
      }
    }
  )
}
// Данные конвертации взяты из http://docs.oracle.com/database/122/LNOCI/data-types.htm#LNOCI16271

// num_from!(f32, FLT, BFLOAT);
// num_from!(f64, FLT, BDOUBLE);

// Чтобы оракл поместил данные в буфер в этих форматах, ему нужно при define-е указать соответствующую
// длину переменной, а сейчас там всегда указывается длина столбца. Таким образом, оракл всегда будет
// возвращать данные в VNU формате
num_from!(   i8, NumberFlag::Signed, INT);
num_from!(  i16, NumberFlag::Signed, INT);
num_from!(  i32, NumberFlag::Signed, INT);
num_from!(  i64, NumberFlag::Signed, INT);
num_from!(isize, NumberFlag::Signed, INT);

num_from!(   u8, NumberFlag::Unsigned, UIN);
num_from!(  u16, NumberFlag::Unsigned, UIN);
num_from!(  u32, NumberFlag::Unsigned, UIN);
num_from!(  u64, NumberFlag::Unsigned, UIN);
num_from!(usize, NumberFlag::Unsigned, UIN);