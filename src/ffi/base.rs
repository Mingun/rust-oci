use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_int, c_void, c_char, c_uchar, c_uint};
use std::mem;
use std::ptr;
use std::slice;
use num_integer::Integer;

use Result;
use error::{DbError, Error};

use super::native::{HandleType, ErrorHandle};
use super::native::OCIError;
use super::native::{OCIHandleAlloc, OCIHandleFree, OCIAttrGet, OCIAttrSet, OCIErrorGet};
use super::Env;
use super::types;

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
pub fn check(native: c_int) -> Result<()> {
  return match native {
    0 => Ok(()),
    e => Err(Error::Db(DbError::Unknown(e as isize)))
  };
}
//-------------------------------------------------------------------------------------------------
/// Типаж, позволяющий получать и устанавливать атрибуты тем структурам, которые его реализуют.
pub trait AttrHolder<T> {
  /// Тип владельца атрибутов
  fn holder_type() -> c_uint;

  fn native(&self) -> *const T;

  fn native_mut(&mut self) -> *mut T {
    self.native() as *mut T
  }

  /// Получает значение указанного атрибута из объекта-владельца атрибутов
  unsafe fn get(&self, value: *mut c_void, size: &mut c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    let res = OCIAttrGet(
      self.native() as *const c_void, Self::holder_type(),
      value, size, attrtype as c_uint,
      err.native_mut()
    );
    return err.check(res);
  }
  fn set(&mut self, value: *mut c_void, size: c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    let res = unsafe {
      OCIAttrSet(
        self.native_mut() as *mut c_void, Self::holder_type(),
        value, size, attrtype as c_uint,
        err.native_mut()
      )
    };
    return err.check(res);
  }

//-------------------------------------------------------------------------------------------------
  fn get_<I: Integer>(&self, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<I> {
    let mut res = I::zero();
    let ptr = &mut res as *mut I;
    try!(unsafe { self.get(ptr as *mut c_void, &mut 0, attrtype, err) });

    Ok(res)
  }
  fn get_str(&self, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<String> {
    let mut len: c_uint = 0;
    let mut str: *mut c_uchar = ptr::null_mut();
    let ptr = &mut str as *mut *mut c_uchar;
    unsafe {
      try!(self.get(ptr as *mut c_void, &mut len, attrtype, err));
      //FIXME: Нужно избавиться от паники, должна возвращаться ошибка
      let cstr = CString::new(slice::from_raw_parts(str, len as usize)).expect("OCIAttrGet call returns string with embedded NUL byte");

      Ok(cstr.into_string().expect("OCIAttrGet call returns non UTF-8 string"))
    }
  }
//-------------------------------------------------------------------------------------------------
  fn set_<I: Integer>(&mut self, value: I, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    let ptr = &value as *const I;
    self.set(ptr as *mut c_void, mem::size_of::<I>() as c_uint, attrtype, err)
  }
  /// Устанавливает строковый атрибут хендлу
  fn set_str(&mut self, value: &str, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    self.set(value.as_ptr() as *mut c_void, value.len() as c_uint, attrtype, err)
  }
  /// Устанавливает хендл-атрибут хендлу
  fn set_handle<U: HandleType>(&mut self, value: &Handle<U>, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    self.set(value.native as *mut c_void, 0, attrtype, err)
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