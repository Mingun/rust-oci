use std::fmt;
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;

use DbResult;
use error::DbError;

use ffi::{check, decode_error, Env};// Основные типобезопасные примитивы
use ffi::{ErrorHandle, HandleType};// Типажи для безопасного моста к FFI

use ffi::attr::AttrHolder;
use ffi::native::OCIError;// FFI типы
use ffi::native::{OCIHandleAlloc, OCIHandleFree};// FFI функции

//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый хендл на ресурсы оракла
pub struct Handle<T: HandleType> {
  native: *mut T,
}
impl<T: HandleType> Handle<T> {
  /// Создает новый хендл в указанном окружении
  ///
  /// # Параметры
  /// - env:
  ///   Окружение, которое будет владеть созданным хендлом
  /// - err:
  ///   Хендл для сбора ошибок при создании хендла. Может отсутствовать (когда создается сам хендл для сбора ошибок)
  pub fn new<E: ErrorHandle>(env: &Env, err: *mut E) -> DbResult<Handle<T>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIHandleAlloc(
        env.native() as *const c_void,
        &mut handle, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделенное под них место
      )
    };
    Self::from_ptr(res, handle as *mut T, err)
  }
  pub fn from_ptr<E: ErrorHandle>(res: c_int, native: *mut T, err: *mut E) -> DbResult<Handle<T>> {
    match res {
      0 => Ok(Handle { native: native }),
      e => Err(decode_error(err, e)),
    }
  }
  #[inline]
  pub fn native_mut(&self) -> *mut T {
    self.native
  }
}
impl<T: HandleType> Drop for Handle<T> {
  fn drop(&mut self) {
    let res = unsafe { OCIHandleFree(self.native as *mut c_void, T::ID as c_uint) };
    //FIXME: Необходимо получать точную причину ошибки, а для этого нужна ссылка на OCIError.
    // Однако тащить ее в хендл нельзя, т.к. данная структура должна быть легкой
    check(res).expect("OCIHandleFree");
  }
}
impl<T: HandleType> fmt::Debug for Handle<T> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("Handle")
       .field(&T::ID)
       .field(&self.native)
       .finish()
  }
}
impl<T: HandleType> AttrHolder<T> for Handle<T> {
  fn holder_type() -> c_uint {
    T::ID as c_uint
  }

  fn native(&self) -> *const T {
    self.native
  }
}

impl Handle<OCIError> {
  /// Транслирует результат, возвращенный любой функцией, в код ошибки базы данных
  pub fn decode(&self, result: c_int) -> DbError {
    decode_error(self.native, result)
  }
  pub fn check(&self, result: c_int) -> DbResult<()> {
    match result {
      0 => Ok(()),
      e => Err(self.decode(e)),
    }
  }
}