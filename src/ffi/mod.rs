
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_int, c_void, c_char, c_uchar, c_uint, c_ushort};
use std::ptr;
use std::slice;
use super::Error;
use super::Result;
use super::ConnectParams;

mod types;
mod native;

pub use self::types::{CreateMode, AttachMode, MallocFn, ReallocFn, FreeFn};
use self::native::*;

//-------------------------------------------------------------------------------------------------
/// Транслирует результат, возвращенный любой функцией, в код ошибки базы данных
///
/// # Параметры
/// - handle:
///   Хендл, и которого можно излечь информацию об ошибке. Обычно это специальный хендл `OCIError`, но
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
      0 as *mut c_uchar,// Устаревший с версии 8.x параметр, не используется
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
fn decode_error<T: ErrorHandle>(handle: *mut T, result: c_int) -> Error {
  let (_, code, msg) = decode_error_piece(handle, 1);
  Error { result: result as isize, code: code as isize, message: msg }
}

//-------------------------------------------------------------------------------------------------
unsafe fn attr_get<T: AttrHandle>(handle: *const T, value: *mut c_void, size: &mut c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
  let res = OCIAttrGet(
    handle as *const c_void, T::ID as c_uint,
    value, size, attrtype as c_uint,
    err.native_mut()
  );
  return err.check(res);
}
fn attr_get_c_uint<T: AttrHandle>(handle: *const T, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<c_uint> {
  let mut res: c_uint = 0;
  let ptr = &mut res as *mut c_uint;
  try!(unsafe { attr_get(handle, ptr as *mut c_void, &mut 0, attrtype, err) });

  Ok(res)
}
unsafe fn attr_set<T: AttrHandle>(handle: *mut T, value: *mut c_void, size: c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
  let res = OCIAttrSet(
    handle as *mut c_void, T::ID as c_uint,
    value, size, attrtype as c_uint,
    err.native_mut()
  );
  return err.check(res);
}
//-------------------------------------------------------------------------------------------------
fn param_get<'d, T: ParamHandle>(handle: *const T, pos: c_uint, err: &Handle<OCIError>) -> Result<Descriptor<'d, OCIParam>> {
  let mut desc = ptr::null_mut();
  let res = unsafe {
    OCIParamGet(
      handle as *const c_void, T::ID as c_uint,
      err.native_mut(),
      &mut desc, pos
    )
  };
  match res {
    0 => Ok(Descriptor { native: desc as *const OCIParam, phantom: PhantomData }),
    e => Err(err.decode(e)),
  }
}
//-------------------------------------------------------------------------------------------------
fn check(native: c_int) -> Result<()> {
  return match native {
    0 => Ok(()),
    e => Err(Error::unknown(e as isize))
  };
}
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый хендл на ресурсы оракла
struct Handle<T: HandleType> {
  native: *mut T,
}
impl<T: HandleType> Handle<T> {
  /// Создает новый хендл в указанном окружении
  ///
  /// # Параметры
  /// - env:
  ///   Окружение, которое будет владеть созданным хендлом
  /// - err:
  ///   Хендл для сюора ошибок при создании хендла. Может отсутствовать (когда создается сам хендл для сбора ошибок)
  fn new<E: ErrorHandle>(env: &Env, err: *mut E) -> Result<Handle<T>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIHandleAlloc(
        env.native as *const c_void,
        &mut handle, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделеное под них место
      )
    };
    return match res {
      0 => Ok(Handle { native: handle as *mut T }),
      e => Err(decode_error(err, e)),
    };
  }
  fn set(&mut self, value: *mut c_void, size: c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    //FIXME: Использование данной функции требует реализовать AttrHandle для всех, кто реализует HandleType.
    // Но к сожалению, попытка сделать это дает невнятную ошибку о том, что тип-донор для ID может не жить достаточно долго(!)
    //unsafe { attr_set(self.native, value, size, attrtype, err) }
    let res = unsafe {
      OCIAttrSet(
        self.native as *mut c_void, T::ID as c_uint,
        value, size, attrtype as c_uint,
        err.native_mut()
      )
    };
    return err.check(res);
  }
  /// Устанавливает строковый атрибут хендлу
  fn set_str(&mut self, value: &str, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    self.set(value.as_ptr() as *mut c_void, value.len() as c_uint, attrtype, err)
  }
  /// Устанавливает хендл-атрибут хендлу
  fn set_handle<U: HandleType>(&mut self, value: &Handle<U>, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    self.set(value.native as *mut c_void, 0, attrtype, err)
  }
  fn native_mut(&self) -> *mut T {
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

impl Handle<OCIError> {
  /// Транслирует результат, возвращенный любой функцией, в код ошибки базы данных
  fn decode(&self, result: c_int) -> Error {
    decode_error(self.native, result)
  }
  fn check(&self, result: c_int) -> Result<()> {
    match result {
      0 => Ok(()),
      e => Err(self.decode(e)),
    }
  }
}
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый дескриптор ресурсов оракла
struct Descriptor<'d, T: 'd + DescriptorType> {
  native: *const T,
  phantom: PhantomData<&'d T>,
}
impl<'d, T: 'd + DescriptorType> Descriptor<'d, T> {
  fn new<'e>(env: &'e Environment) -> Result<Descriptor<'e, T>> {
    let mut desc = ptr::null_mut();
    let res = unsafe {
      OCIDescriptorAlloc(
        env.env.native as *const c_void,
        &mut desc, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделеное под них место
      )
    };
    return match res {
      0 => Ok(Descriptor { native: desc as *const T, phantom: PhantomData }),
      e => Err(env.error.decode(e))
    };
  }
  unsafe fn get(&self, value: *mut c_void, size: &mut c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    //FIXME: Использование данной функции требует реализовать AttrHandle для всех, кто реализует DescriptorType.
    // Но к сожалению, попытка сделать это дает невнятную ошибку о том, что тип-донор для ID может не жить достаточно долго(!)
    //unsafe { attr_get(self.native, value, size, attrtype, err) }
    let res = OCIAttrGet(
      self.native as *const c_void, T::ID as c_uint,
      value, size, attrtype as c_uint,
      err.native_mut()
    );
    return err.check(res);
  }
  unsafe fn set(&self, value: *mut c_void, size: c_uint, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<()> {
    //FIXME: Использование данной функции требует реализовать AttrHandle для всех, кто реализует DescriptorType.
    // Но к сожалению, попытка сделать это дает невнятную ошибку о том, что тип-донор для ID может не жить достаточно долго(!)
    //unsafe { attr_set(self.native, value, size, attrtype, err) }
    let res = OCIAttrSet(
      self.native as *mut c_void, T::ID as c_uint,
      value, size, attrtype as c_uint,
      err.native_mut()
    );
    return err.check(res);
  }
  fn get_c_uint(&self, attrtype: types::Attr, err: &Handle<OCIError>) -> Result<c_uint> {
    let mut res: c_uint = 0;
    let ptr = &mut res as *mut c_uint;
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
}
impl<'d, T: 'd + DescriptorType> Drop for Descriptor<'d, T> {
  fn drop(&mut self) {
    let res = unsafe { OCIDescriptorFree(self.native as *mut c_void, T::ID as c_uint) };
    //FIXME: Необходимо получать точную причину ошибки, а для этого нужна ссылка на OCIError.
    // Однако тащить ее в дескриптор нельзя, т.к. данная структура должна быть легкой
    check(res).expect("OCIDescriptorFree");
  }
}
impl<'d, T: 'd + DescriptorType> fmt::Debug for Descriptor<'d, T> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_tuple("Descriptor")
       .field(&T::ID)
       .field(&self.native)
       .finish()
  }
}

//-------------------------------------------------------------------------------------------------
/// Автоматически закрываемый хендл окружения оракла
#[derive(Debug)]
struct Env<'e> {
  native: *const OCIEnv,
  mode: types::CreateMode,
  /// Фантомные данные для статического анализа управления временем жизни окружения. Эмулирует владение
  /// указателем `native` структуры.
  phantom: PhantomData<&'e OCIEnv>,
}
impl<'e> Env<'e> {
  fn new(mode: types::CreateMode) -> Result<Self> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIEnvNlsCreate(
        &mut handle, // сюда записывается результат
        mode as c_uint,
        0 as *mut c_void, // Контекст для функций управления памятью.
        None, None, None, // Функции управления памятью
        0, 0 as *mut *mut c_void,// размер пользовательских данных и указатель на выделеное под них место
        0, 0// Параметры локализации для типов CHAR и NCHAR. 0 - использовать настройку NLS_LANG
      )
    };
    return match res {
      0 => Ok(Env { native: handle, mode: mode, phantom: PhantomData }),
      // Ошибки создания окружения никуда не записываются, т.к. им просто некуда еще записываться
      e => Err(Error::unknown(e as isize))
    };
  }
  /// Создает новый хендл в указанном окружении запрашиваемого типа
  /// # Параметры
  /// - err:
  ///   Хендл для сбора ошибок, куда будет записана ошибка в случае, если создание хендла окажется неудачным
  fn handle<T: HandleType, E: ErrorHandle>(&self, err: *mut E) -> Result<Handle<T>> {
    Handle::new(&self, err)
  }
}
impl<'e> Drop for Env<'e> {
  fn drop(&mut self) {
    let res = unsafe { OCITerminate(self.mode as c_uint) };
    // Получить точную причину ошибки в этом месте нельзя, т.к. все структуры уже разрушены
    check(res).expect("OCITerminate");
  }
}
//-------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Environment<'e> {
  env: Env<'e>,
  error: Handle<OCIError>,
}
impl<'e> Environment<'e> {
  pub fn new(mode: types::CreateMode) -> Result<Self> {
    let env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.handle(env.native as *mut OCIEnv));

    Ok(Environment { env: env, error: err })
  }
  pub fn connect<P: Into<ConnectParams>>(&self, params: P) -> Result<Connection> {
    let p = params.into();
    Connection::new(&self, &p.dblink, p.mode, &p.username, &p.password)
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    self.env.handle(self.error.native)
  }
  fn descriptor<T: DescriptorType>(&self) -> Result<Descriptor<T>> {
    Descriptor::new(&self)
  }
  fn error(&self) -> &Handle<OCIError> {
    &self.error
  }
}
//-------------------------------------------------------------------------------------------------
/// Хранит автоматически закрываемый хендл `OCIServer`, предоставляющий доступ к базе данных
#[derive(Debug)]
struct Server<'env> {
  env: &'env Environment<'env>,
  handle: Handle<OCIServer>,
  mode: types::AttachMode,
}
impl<'env> Server<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode) -> Result<Server<'e>> {
    let server: Handle<OCIServer> = try!(env.handle());
    let res = unsafe {
      OCIServerAttach(
        server.native_mut(), env.error.native_mut(),
        dblink.as_ptr(), dblink.len() as c_int,
        mode as c_uint
      )
    };
    return match res {
      0 => Ok(Server { env: env, handle: server, mode: mode }),
      e => Err(env.error.decode(e))
    };
  }
  fn error(&self) -> &Handle<OCIError> {
    self.env.error()
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
//-------------------------------------------------------------------------------------------------
/// Представляет соединение к базе данных, с определенным пользователем и паролем.
/// Соединение зависит от окружения, создавшего его, таким образом, окружение является менеджером
/// соединений. При уничтожении окружения все соединения закрываются, а незакоммиченные транзакции
/// в них откатываются.
#[derive(Debug)]
pub struct Connection<'env> {
  server: Server<'env>,
  context: Handle<OCISvcCtx>,
  session: Handle<OCISession>,
}
impl<'env> Connection<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode, username: &str, password: &str) -> Result<Connection<'e>> {
    let server = try!(Server::new(env, dblink, mode));
    let mut context: Handle<OCISvcCtx > = try!(env.handle());
    let mut session: Handle<OCISession> = try!(env.handle());

    // Ассоциируем имя пользователя и пароль с сессией
    try!(session.set_str(username, types::Attr::Username, &env.error));
    try!(session.set_str(password, types::Attr::Password, &env.error));

    // Ассоциируем сервер с контекстом и осуществляем подключение
    try!(context.set_handle(&server.handle, types::Attr::Server, &env.error));
    let res = unsafe {
      OCISessionBegin(
        context.native_mut(),
        env.error.native_mut(),
        session.native_mut(),
        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        types::CredentialMode::Rdbms as c_uint,
        types::AuthMode::Default as c_uint
      )
    };
    try!(env.error.check(res));
    try!(context.set_handle(&session, types::Attr::Session, &env.error));

    Ok(Connection { server: server, context: context, session: session })
  }
  fn error(&self) -> &Handle<OCIError> {
    self.server.error()
  }

  pub fn prepare<'c>(&'c self, sql: &str) -> Result<Statement<'c, 'c>> {
    Statement::new(&self, sql, None, types::Syntax::Native)
  }
}
impl<'env> Drop for Connection<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCISessionEnd(
        self.context.native_mut(),
        self.error().native_mut(),
        self.session.native_mut(),
        types::AuthMode::Default as c_uint
      )
    };
    self.error().check(res).expect("OCISessionEnd");
  }
}
//-------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Statement<'conn, 'key> {
  /// Соединение, которое подготовило данное выражение
  conn: &'conn Connection<'conn>,
  /// Внутренний указатель оракла на подготовленное выражение
  native: *const OCIStmt,
  /// Ключ для кеширования выражения
  key: Option<&'key str>,
}
impl<'conn, 'key> Statement<'conn, 'key> {
  fn new<'c, 'k>(conn: &'c Connection<'c>, sql: &str, key: Option<&'k str>, syntax: types::Syntax) -> Result<Statement<'c, 'k>> {
    let mut stmt = ptr::null_mut();
    let keyPtr = key.map_or(0 as *const c_uchar, |x| x.as_ptr() as *const c_uchar);
    let keyLen = key.map_or(0 as c_uint        , |x| x.len()  as c_uint);
    let res = unsafe {
      OCIStmtPrepare2(
        conn.context.native_mut(),
        &mut stmt as *mut *mut OCIStmt,
        conn.error().native_mut(),
        // Текст SQL запроса
        sql.as_ptr() as *const c_uchar, sql.len() as c_uint,
        // Ключ кеширования, по которому достанется запрос, если он был закеширован
        keyPtr, keyLen,
        syntax as c_uint, types::CachingMode::Default as c_uint
      )
    };
    return match res {
      0 => Ok(Statement { conn: conn, native: stmt, key: key }),
      e => Err(conn.error().decode(e)),
    };
  }
  fn error(&self) -> &Handle<OCIError> {
    self.conn.error()
  }
  /// # Параметры
  /// - count:
  ///   * Для `select` выражений это количество строк, которые нужно извлечть prefetch-ем, уже в момент выполнения
  ///     запроса (т.е. сервер БД вернет их, не дожидаясь вызова `OCIStmtFetch2`). Если prefetch не нужен, то должно
  ///     быть равно `0`.
  ///   * Для не-`select` выражений это номер последнего элемента в буфере данных со связанными параметрами, которые
  ///     нужно использовать при выполнении данной операции
  /// - offset:
  ///   Смещение с буфере со связанными переменными, с которого необходимо начать выполнение 
  fn execute(&self, count: c_uint, offset: c_uint, mode: types::ExecuteMode) -> Result<()> {
    let res = unsafe {
      OCIStmtExecute(
        self.conn.context.native_mut(),
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        offset,
        ptr::null(),
        ptr::null_mut(),
        mode as c_uint
      )
    };
    return self.error().check(res);
  }
  /// Извлекает из текущего выражения данные, которые в нем имеются после выполнения `select`-а.
  ///
  /// # Параметры
  /// - count:
  ///   Количество строк, которые нужно получить из текущей позиции курсора
  /// - index:
  ///   Для режимов `Absolute` и `Relative` определяет номер извлекаемого элемента, в остальных случаях игнорируется.
  fn fetch(&self, count: c_uint, mode: types::FetchMode, index: c_int) -> Result<()> {
    let res = unsafe {
      OCIStmtFetch2(
        self.native as *mut OCIStmt,
        self.error().native_mut(),
        count,
        mode as c_ushort,
        index as c_int,
        0 // Неясно, что такое
      )
    };
    return self.error().check(res);
  }
  fn param_count(&self) -> Result<c_uint> {
    attr_get_c_uint(self.native, types::Attr::ParamCount, self.error())
  }
  fn param_get(&self, pos: c_uint) -> Result<Descriptor<OCIParam>> {
    param_get(self.native, pos, self.error())
  }

  pub fn columns(&self) -> Result<Vec<Column>> {
    let cnt = try!(self.param_count());
    let mut vec = Vec::with_capacity(cnt as usize);
    for i in 0..cnt {
      vec.push(try!(Column::new(i as usize, try!(self.param_get(i+1)), self.error())));
    }
    Ok(vec)
  }
  pub fn query(&self) -> Result<()> {
    self.execute(0, 0, Default::default())
  }
}
impl<'conn, 'key> Drop for Statement<'conn, 'key> {
  fn drop(&mut self) {
    let keyPtr = self.key.map_or(0 as *const c_uchar, |x| x.as_ptr() as *const c_uchar);
    let keyLen = self.key.map_or(0 as c_uint        , |x| x.len()  as c_uint);
    let res = unsafe { OCIStmtRelease(self.native as *mut OCIStmt, self.error().native_mut(), keyPtr, keyLen, 0) };
    self.error().check(res).expect("OCIStmtRelease");
  }
}

//-------------------------------------------------------------------------------------------------
/// Структура для представления колонки базы данных из списка выбора
#[derive(Debug)]
pub struct Column {
  /// Номер столбца
  pub pos: usize,
  pub type_: types::Type,
  /// Название колонки в базе данных
  pub name: String,
  /// Ширина колонки в байтах
  pub size: usize,
}

impl Column {
  fn new(pos: usize, desc: Descriptor<OCIParam>, err: &Handle<OCIError>) -> Result<Self> {
    let type_ = try!(desc.get_c_uint(types::Attr::DataType, err));
    let name  = try!(desc.get_str(types::Attr::Name, err));
    //let ischar= try!(desc.get_c_uint(types::Attr::CharUsed, err));
    //let size  = try!(desc.get_c_uint(types::Attr::CharSize, err));
    let size  = try!(desc.get_c_uint(types::Attr::DataSize, err));

    Ok(Column { pos: pos, name: name, size: size as usize, type_: unsafe { mem::transmute(type_ as u16) } })
  }
}