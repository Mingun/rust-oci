
use std::convert::{From, Into};
use std::mem;
use std::os::raw::{c_int, c_short, c_void, c_ushort};
use std::ptr;
use std::slice;

use {Connection, Statement, Result};
use stmt::Column;
use types::{FromDB, Type};

use ffi::{Descriptor, GenericDescriptor};// Основные типобезопасные примитивы
use ffi::DescriptorType;// Типажи для безопасного моста к FFI

use ffi::native::time::{Timestamp, TimestampWithTZ, TimestampWithLTZ, IntervalYM, IntervalDS};
use ffi::native::lob::{Lob, File};

#[derive(Debug)]
pub enum Storage<'d> {
  Vec {
    /// Указатель на начало памяти, где будут храниться данные
    ptr: *mut u8,
    /// Количество байт, выделенной по указателю `ptr`.
    capacity: usize,
    /// Количество байт, реально используемое для хранения данных.
    size: c_ushort,
  },
  Descriptor(GenericDescriptor<'d>),
}
impl<'d> Storage<'d> {
  /// Получает адрес блока памяти, который можно использовать для записи в него значений
  fn as_ptr(&mut self) -> *mut c_void {
    match *self {
      Storage::Vec { ptr, .. } => ptr as *mut c_void,
      Storage::Descriptor(ref mut d) => d.address_mut(),
    }
  }
  /// Получает вместимость буфера
  fn capacity(&self) -> c_int {
    match *self {
      Storage::Vec { capacity, .. } => capacity as c_int,
      _ => mem::size_of::<*const ()>() as c_int,
    }
  }
  /// Получает адрес в памяти, куда будет записан размер данных, фактически извлеченный из базы
  fn size_mut(&mut self) -> *mut c_ushort {
    match *self {
      Storage::Vec { ref mut size, .. } => size,
      _ => ptr::null_mut(),
    }
  }
  fn as_slice(&self) -> &[u8] {
    match *self {
      Storage::Vec { ptr, size, .. } => unsafe { slice::from_raw_parts(ptr, size as usize) },
      Storage::Descriptor(ref d) => d.as_slice(),
    }
  }
}
impl<'d> From<Vec<u8>> for Storage<'d> {
  fn from(mut backend: Vec<u8>) -> Self {
    let res = Storage::Vec { ptr: backend.as_mut_ptr(), size: 0, capacity: backend.capacity() };
    // Вектор уходит в небытие, чтобы он не забрал память с собой, забываем его
    mem::forget(backend);
    res
  }
}
impl<'d, T: DescriptorType> From<Descriptor<'d, T>> for Storage<'d> {
  fn from(backend: Descriptor<'d, T>) -> Self {
    Storage::Descriptor(backend.into())
  }
}
impl<'d> Drop for Storage<'d> {
  fn drop(&mut self) {
    // Освобождаем память деструктором вектора, ведь память была выделена его конструктором
    if let Storage::Vec { ptr, capacity, size } = *self {
      unsafe { Vec::from_raw_parts(ptr, size as usize, capacity) };
    };
  }
}

macro_rules! alloc {
  (
    $stmt:expr, $col:expr,
    $($kind:ident, $ty:ty),*
  ) => (
    match $col.type_ {
      $(
        Type::$kind => {
          let d: Descriptor<'d, $ty> = try!($stmt.conn.server.new_descriptor());
          Ok(d.into())
        }
      )*
      _ => Ok(Vec::with_capacity($col.size).into()),
    }
  );
}
/// Хранилище буферов для биндинга результатов, извлекаемых из базы, для одной колонки
#[derive(Debug)]
pub struct DefineInfo<'d> {
  storage: Storage<'d>,
  /// Возможные значения:
  /// * `-2`  The length of the item is greater than the length of the output variable; the item has been truncated. Additionally,
  ///         the original length is longer than the maximum data length that can be returned in the sb2 indicator variable.
  /// * `-1`  The selected value is null, and the value of the output variable is unchanged.
  /// * `0`   Oracle Database assigned an intact value to the host variable.
  /// * `>0`  The length of the item is greater than the length of the output variable; the item has been truncated. The positive
  ///         value returned in the indicator variable is the actual length before truncation.
  pub is_null: c_short,
  pub ret_code: c_ushort,
}
impl<'d> DefineInfo<'d> {
  /// Создает буферы для хранения информации, извлекаемой из базы
  pub fn new(stmt: &'d Statement, column: &Column) -> Result<Self> {
    alloc!(stmt, column,
      TIMESTAMP, Timestamp,
      TIMESTAMP_TZ, TimestampWithTZ,
      TIMESTAMP_LTZ, TimestampWithLTZ,

      INTERVAL_YM, IntervalYM,
      INTERVAL_DS, IntervalDS,

      CLOB, Lob,
      BLOB, Lob,
      BFILEE, File,
      CFILEE, File
    )
  }
  #[inline]
  pub fn as_ptr(&mut self) -> *mut c_void {
    self.storage.as_ptr()
  }
  #[inline]
  pub fn capacity(&self) -> c_int {
    self.storage.capacity()
  }
  #[inline]
  pub fn size_mut(&mut self) -> *mut c_ushort {
    self.storage.size_mut()
  }

  /// Возвращает представление данного хранилища в виде среза из массива байт, если
  /// в хранилище есть данные и `None`, если в хранилище хранится `NULL` значение.
  #[inline]
  fn as_slice(&self) -> Option<&[u8]> {
    match self.is_null {
      0 => Some(self.storage.as_slice()),
      _ => None
    }
  }
  /// Представляет содержимое данного хранилища в виде объекта указанного типа
  #[inline]
  pub fn to<T: FromDB>(&self, ty: Type, conn: &Connection) -> Result<Option<T>> {
    match self.as_slice() {
      Some(ref slice) => T::from_db(ty, slice, conn).map(|r| Some(r)),
      None => Ok(None),
    }
  }
}
impl<'d> From<Vec<u8>> for DefineInfo<'d> {
  fn from(backend: Vec<u8>) -> Self {
    DefineInfo { storage: backend.into(), is_null: 0, ret_code: 0 }
  }
}
impl<'d, T> From<Descriptor<'d, T>> for DefineInfo<'d>
  where T: DescriptorType,
        Storage<'d>: From<Descriptor<'d, T>>
{
  fn from(backend: Descriptor<'d, T>) -> Self {
    DefineInfo { storage: backend.into(), is_null: 0, ret_code: 0 }
  }
}