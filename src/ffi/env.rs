use std::fmt;
use std::marker::PhantomData;
use std::os::raw::{c_uint, c_void};
use std::ptr;

use Result;
use error::{DbError, Error};
use types::CreateMode;

use super::{check, Handle};
use super::native::{OCIEnv, OCIError};// FFI типы
use super::native::{OCIEnvNlsCreate, OCITerminate};// FFI функции
use super::native::{ErrorHandle, HandleType};// Типажи для безопасного моста к FFI
//-------------------------------------------------------------------------------------------------
/// Автоматически закрываемый хендл окружения оракла
pub struct Env<'e> {
  native: *const OCIEnv,
  mode: CreateMode,
  /// Фантомные данные для статического анализа управления временем жизни окружения. Эмулирует владение
  /// указателем `native` структуры.
  phantom: PhantomData<&'e OCIEnv>,
}
impl<'e> Env<'e> {
  pub fn new(mode: CreateMode) -> Result<Self> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIEnvNlsCreate(
        &mut handle, // сюда записывается результат
        mode as c_uint,
        0 as *mut c_void, // Контекст для функций управления памятью.
        None, None, None, // Функции управления памятью
        0, 0 as *mut *mut c_void,// размер пользовательских данных и указатель на выделенное под них место
        0, 0// Параметры локализации для типов CHAR и NCHAR. 0 - использовать настройку NLS_LANG
      )
    };
    return match res {
      0 => Ok(Env { native: handle, mode: mode, phantom: PhantomData }),
      // Ошибки создания окружения никуда не записываются, т.к. им просто некуда еще записываться
      e => Err(Error::Db(DbError::Unknown(e as isize)))
    };
  }
  /// Создает новый хендл в указанном окружении запрашиваемого типа
  ///
  /// # Параметры
  /// - err:
  ///   Хендл для сбора ошибок, куда будет записана ошибка в случае, если создание хендла окажется неудачным
  #[inline]
  pub fn new_handle<T: HandleType, E: ErrorHandle>(&self, err: *mut E) -> Result<Handle<T>> {
    Handle::new(&self, err)
  }
  #[inline]
  pub fn new_error_handle(&mut self) -> Result<Handle<OCIError>> {
    self.new_handle(self.native as *mut OCIEnv)
  }
  /// Получает голый указатель на хендл окружения, используемый для передачи в нативные функции.
  #[inline]
  pub fn native(&self) -> *const OCIEnv {
    self.native
  }
  /// Данная функция существует по той причине, что ее вызов при разрушения данного объекта приведет к невозможности заново создать
  /// данный объект, т.к. повторная инициализация окружения вызывает crash в недрах OCI. По этому поводу еще с 2015 года [есть вопрос][1]
  /// на официальном форуме сообщества Oracle, который был проигнорирован.
  ///
  /// Не рекомендуется вызывать данную функцию. При выгрузки приложения из памяти операционная система в любом случае почистит
  /// все неосвобожденные ресурсы. Также в примерах Oracle-а данная функция не вызывается
  ///
  /// # OCI вызовы
  /// Выполняет OCI вызов [`OCITerminate()`][end].
  ///
  /// # Запросы к серверу (1)
  /// Завершение работы требует посылки запроса на сервер. Это выглядит немного страно, учитывая, что создание окружения никаких
  /// запросов не посылает.
  ///
  /// [1]: https://community.oracle.com/thread/3779405
  /// [end]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17127
  #[deprecated(note = "Calling of this function will result in impossibility to anew initialize oci because of crash: https://community.oracle.com/thread/3779405")]
  pub fn terminate() -> Result<()> {
    let res = unsafe { OCITerminate(0) };
    // Получить точную причину ошибки в этом месте нельзя, т.к. все структуры уже разрушены
    check(res)
  }
}
impl<'e> fmt::Debug for Env<'e> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("Env")
       .field(&self.native)
       .field(&self.mode)
       .finish()
  }
}