//! Содержит код для преобразованием между типами Rust и типами базы данных.

use std::os::raw::c_void;
use std::marker::PhantomData;
use std::ptr;
use std::str;
use std::time::Duration;

use {Connection, Result};
use error::Error;
use types::Type;

pub use self::num::OCINumber;

mod num;
mod bin;
#[cfg(feature = "with-chrono")]
mod chrono;

/// Преобразует тип базы данных в тип Rust, для которого реализован данный типаж.
pub trait FromDB<'conn> : 'conn + Sized {
  /// Преобразует данные, извлеченные из базы данных, в конкретный тип, если это преобразование
  /// возможно. Если преобразование невозможно, или в процессе преобразования возникает ошибка,
  /// возвращает `Err`.
  ///
  /// # Параметры
  /// - `ty`:
  ///   Специфический для базы тип данных, из которого требуется сделать преобразование
  /// - `raw`:
  ///   Слепок данных для значения указанного типа, которое необходимо преобразовать в Rust-тип.
  /// - `conn`:
  ///   Соединение, в рамках которого было выполнено выражение, извлекшее представленные данные.
  fn from_db(ty: Type, raw: &[u8], conn: &'conn Connection) -> Result<Self>;
}

impl<'conn> FromDB<'conn> for String {
  fn from_db(ty: Type, raw: &[u8], _: &Connection) -> Result<Self> {
    match ty {
      Type::CHR |
      Type::AFC => str::from_utf8(raw).map(str::to_owned).map_err(|_| Error::Conversion(Type::CHR)),
      t => Err(Error::Conversion(t)),
    }
  }
}

use ffi::native::time::{get_day_second, IntervalDS};

impl<'conn> FromDB<'conn> for Duration {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::INTERVAL_DS => {
        from_ds(ty, raw, conn)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
fn from_ds(ty: Type, raw: &[u8], conn: &Connection) -> Result<Duration> {
  let i: &IntervalDS = unsafe { conn.as_descriptor(raw) };
  let dur = try!(get_day_second(&conn.session, conn.error(), i));

  if dur[0] < 0
  || dur[1] < 0
  || dur[2] < 0
  || dur[3] < 0
  || dur[4] < 0 {
    return Err(Error::Conversion(ty));
  }
  let dd = dur[0] as u64;
  let hh = dur[1] as u64;
  let mm = dur[2] as u64;
  let ss = dur[3] as u64;
  let ns = dur[4] as u32;
  let secs = ((dd*24 + hh)*60 + mm)*60 + ss;
  Ok(Duration::new(secs, ns))
}
//-------------------------------------------------------------------------------------------------

/// Содержит информацию, необходимую для обобщенного связывания любого типа, реализующего `Into<BindInfo>`.
#[derive(Debug)]
pub struct BindInfo<'a> {
  /// Указатель на начало памяти, содержащей данные для связывания.
  pub ptr: *const c_void,
  /// Размер данных, на которые указывает `ptr`.
  pub size: usize,
  /// Тип базы данных, представленный данной структурой.
  pub ty: Type,
  /// Признак того, что переменная связывания содержит `NULL`.
  pub is_null: i16,
  /// Маркер, привязывающей структуре время жизни.
  pub _phantom: PhantomData<&'a ()>,
}
impl<'a> BindInfo<'a> {
  #[inline]
  fn from_slice(slice: &[u8], ty: Type) -> Self {
    BindInfo {
      ptr: slice.as_ptr() as *const c_void,
      size: slice.len(),
      ty: ty,
      is_null: 0,
      _phantom: PhantomData,
    }
  }
  #[inline]
  fn null(ty: Type) -> Self {
    BindInfo {
      ptr: ptr::null(),
      size: 0,
      ty: ty,
      is_null: -1,
      _phantom: PhantomData,
    }
  }
}
impl<'a> Into<BindInfo<'a>> for &'a str {
  fn into(self) -> BindInfo<'a> {
    BindInfo::from_slice(self.as_bytes(), Type::CHR)
  }
}
impl<'a> Into<BindInfo<'a>> for &'a String {
  fn into(self) -> BindInfo<'a> {
    BindInfo::from_slice(self.as_bytes(), Type::CHR)
  }
}
