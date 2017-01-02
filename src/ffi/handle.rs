use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_uchar, c_int, c_uint, c_void};
use std::ptr;

use Result;
use error::{Error, DbError};

use super::Env;
use super::base::AttrHolder;
use super::base::check;
use super::native::OCIError;// FFI типы
use super::native::{OCIHandleAlloc, OCIHandleFree, OCIErrorGet};// FFI функции
use super::native::{ErrorHandle, HandleType};// Типажи для безопасного моста к FFI

//-------------------------------------------------------------------------------------------------
/// Транслирует результат, возвращенный любой функцией, в код ошибки базы данных
///
/// # Параметры
/// - handle:
///   Хендл, и которого можно извлечь информацию об ошибке. Обычно это специальный хендл `OCIError`, но
///   в тех случаях, когда его нет (создание этого хендла ошибки и, почему-то, окружения), можно использовать
///   хендл окружения `OCIEnv`
/// - error_no:
///   Вызовы функций могут возвращать множество ошибок. Это получаемый номер ошибки
/// - msg:
///   Буфер, куда будет записано сообщение оракла об ошибке
fn decode_error_piece<T: ErrorHandle>(handle: *mut T, error_no: c_uint) -> (c_int, c_int, String) {
  let mut code: c_int = 0;
  // Сообщение получается в кодировке, которую установили для хендла окружения.
  // Оракл рекомендует использовать буфер величиной 3072 байта
  let mut buf: Vec<u8> = Vec::with_capacity(3072);
  let res = unsafe {
    OCIErrorGet(
      handle as *mut c_void,
      error_no,
      ptr::null_mut(),// Устаревший с версии 8.x параметр, не используется
      &mut code,
      buf.as_mut_ptr() as *mut c_uchar,
      buf.capacity() as c_uint,
      T::ID as c_uint
    )
  };
  unsafe {
    // Так как функция только заполняет массив, но не возвращает длину, ее нужно вычислить и задать,
    // иначе трансформация в строку ничего не даст, т.к. будет считать массив пустым.
    let msg = CStr::from_ptr(buf.as_ptr() as *const c_char);
    buf.set_len(msg.to_bytes().len());
  };

  (res, code, String::from_utf8(buf).expect("Invalid UTF-8 from OCIErrorGet"))
}
fn decode_error<T: ErrorHandle>(handle: Option<*mut T>, result: c_int) -> DbError {
  match result {
    // Относительный успех
    0 => unreachable!(),// Сюда не должны попадать
    1 => DbError::Info,//TODO: получить диагностическую информацию
    99 => DbError::NeedData,
    100 => DbError::NoData,

    // Ошибки
    -1 => {
      let (_, code, msg) = match handle {
        None => (0, 0, String::new()),
        Some(h) => decode_error_piece(h, 1),
      };
      DbError::Fault { code: code as isize, message: msg }
    },
    -2 => DbError::InvalidHandle,
    -3123 => DbError::StillExecuting,
    e => DbError::Unknown(e as isize),
  }
}
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
  pub fn new<E: ErrorHandle>(env: &Env, err: *mut E) -> Result<Handle<T>> {
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
  pub fn from_ptr<E: ErrorHandle>(res: c_int, native: *mut T, err: *mut E) -> Result<Handle<T>> {
    match res {
      0 => Ok(Handle { native: native }),
      e => Err(Error::Db(decode_error(Some(err), e))),
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
  pub fn decode(&self, result: c_int) -> Error {
    Error::Db(decode_error(Some(self.native), result))
  }
  pub fn check(&self, result: c_int) -> Result<()> {
    match result {
      0 => Ok(()),
      e => Err(self.decode(e)),
    }
  }
}