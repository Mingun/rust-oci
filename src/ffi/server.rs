use std::os::raw::{c_int, c_uint};
use std::ptr;

use {Environment, Result};
use types::AttachMode;

use super::{Handle, Descriptor};
use super::native::{OCIServer, OCIError};// FFI типы
use super::native::{OCIServerAttach, OCIServerDetach};// FFI функции
use super::native::DescriptorType;// Типажи для безопасного моста к FFI
//-------------------------------------------------------------------------------------------------
/// Хранит автоматически закрываемый хендл `OCIServer`, предоставляющий доступ к базе данных
#[derive(Debug)]
pub struct Server<'env> {
  env: &'env Environment<'env>,
  handle: Handle<OCIServer>,
  /// Режим создания соединений, установленный при установлении соединения к серверу.
  mode: AttachMode,
}
impl<'env> Server<'env> {
  /// Осуществляет подключение к указанному серверу в рамках данного окружения
  pub fn new(env: &'env Environment, dblink: Option<&str>, mode: AttachMode) -> Result<Self> {
    let server: Handle<OCIServer> = try!(env.handle());
    let (ptr, len) = match dblink {
      Some(db) => (db.as_ptr(), db.len()),
      None => (ptr::null(), 0)
    };
    let res = unsafe {
      OCIServerAttach(
        server.native_mut(), env.error.native_mut(),
        ptr, len as c_int,
        mode as c_uint
      )
    };
    return match res {
      0 => Ok(Server { env: env, handle: server, mode: mode }),
      e => Err(env.error.decode(e))
    };
  }
  /// Получает хендл для записи ошибок во время общения с базой данных. Хендл берется из окружения, которое породило
  /// данный сервер. В случае возникновения ошибки при вызове FFI-функции она может быть получена из хендла с помощью
  /// вызова `decode(ffi_result)`.
  #[inline]
  pub fn error(&self) -> &Handle<OCIError> {
    self.env.error()
  }
  #[inline]
  pub fn descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    self.env.descriptor()
  }
  #[inline]
  pub fn handle(&self) -> &Handle<OCIServer> {
    &self.handle
  }
}
impl<'env> Drop for Server<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCIServerDetach(
        self.handle.native_mut(),
        self.error().native_mut(),
        self.mode as c_uint
      )
    };
    self.error().check(res).expect("OCIServerDetach");
  }
}