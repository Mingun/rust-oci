//! Содержит реализацию автоматически закрываемого хендла окружения
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use DbResult;
use params::InitParams;

use ffi::{check, decode_error, Handle};// Основные типобезопасные примитивы
use ffi::{ErrorHandle, HandleType};// Типажи для безопасного моста к FFI

use ffi::native::{OCIEnv, OCIError};// FFI типы
use ffi::native::{OCIEnvNlsCreate, OCITerminate};// FFI функции

//-------------------------------------------------------------------------------------------------
/// Автоматически закрываемый хендл окружения оракла
pub struct Env<'e> {
  /// Указатель на хендл, полученный от FFI функций оракла.
  native: *const OCIEnv,
  /// Параметры, с которыми было инициализировано данное окружение.
  params: InitParams,
  /// Фантомные данные для статического анализа управления временем жизни окружения. Эмулирует владение
  /// указателем `native` структуры.
  phantom: PhantomData<&'e OCIEnv>,
}
impl<'e> Env<'e> {
  pub fn new(params: InitParams) -> DbResult<Self> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIEnvNlsCreate(
        &mut handle, // сюда записывается результат
        params.mode as u32,
        ptr::null_mut(), // Контекст для функций управления памятью.
        None, None, None, // Функции управления памятью
        0, ptr::null_mut(),// размер пользовательских данных и указатель на выделенное под них место
        // Параметры локализации для типов CHAR и NCHAR
        params.charset as u16, params.ncharset as u16
      )
    };
    return match res {
      0 => Ok(Env { native: handle, params: params, phantom: PhantomData }),
      e => Err(decode_error(handle, e))
    };
  }
  /// Создает новый хендл в указанном окружении запрашиваемого типа
  ///
  /// # Параметры
  /// - err:
  ///   Хендл для сбора ошибок, куда будет записана ошибка в случае, если создание хендла окажется неудачным
  #[inline]
  pub fn new_handle<T: HandleType, E: ErrorHandle>(&self, err: *mut E) -> DbResult<Handle<T>> {
    Handle::new(&self, err)
  }
  #[inline]
  pub fn new_error_handle(&mut self) -> DbResult<Handle<OCIError>> {
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
  /// Завершение работы требует посылки запроса на сервер. Это выглядит немного странно, учитывая, что создание окружения никаких
  /// запросов не посылает.
  ///
  /// [1]: https://community.oracle.com/thread/3779405
  /// [end]: http://docs.oracle.com/database/122/LNOCI/connect-authorize-and-initialize-functions.htm#LNOCI17127
  #[deprecated(note = "Calling of this function will result in impossibility to anew initialize oci because of crash: https://community.oracle.com/thread/3779405")]
  pub fn terminate() -> DbResult<()> {
    let res = unsafe { OCITerminate(0) };
    // Получить точную причину ошибки в этом месте нельзя, т.к. все структуры уже разрушены
    check(res)
  }
}
impl<'e> fmt::Debug for Env<'e> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("Env")
       .field(&self.native)
       .field(&self.params)
       .finish()
  }
}
impl<'e> Default for Env<'e> {
  /// Создает окружение с использованеим параметров по умочланию.
  fn default() -> Self {
    Env::new(Default::default()).expect("Can't create environment with default parameters")
  }
}