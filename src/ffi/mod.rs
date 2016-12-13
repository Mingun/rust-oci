
use std::os::raw::{c_int, c_void, c_uchar, c_uint, c_ushort};
use std::ptr;
use super::Error;
use super::Result;
use super::ConnectParams;

mod types;
mod stmt;

pub use self::types::{CreateMode, AttachMode, MallocFn, ReallocFn, FreeFn};
pub use self::stmt::*;

trait HandleType {
  const ID: types::Handle;
}

enum OCIEnv {}
enum OCIError {}    impl HandleType for OCIError   { const ID: types::Handle = types::Handle::Error; }
enum OCIServer {}   impl HandleType for OCIServer  { const ID: types::Handle = types::Handle::Server; }
enum OCISvcCtx {}   impl HandleType for OCISvcCtx  { const ID: types::Handle = types::Handle::SvcCtx; }
enum OCISession {}  impl HandleType for OCISession { const ID: types::Handle = types::Handle::Session; }

#[link(name = "oci")]
#[allow(dead_code)]
extern "C" {
  /// OCI ENVironment CREATE with NLS info.
  ///
  /// This function does almost everything `OCIEnvCreate` does, plus enabling setting
  /// of charset and ncharset programmatically, except `OCI_UTF16` mode.
  ///
  /// # Comments
  /// The charset and ncharset must be both zero or non-zero.
  /// The parameters have the same meaning as the ones in `OCIEnvCreate()`.
  /// When charset or ncharset is non-zero, the corresponding character set will
  /// be used to replace the ones specified in `NLS_LANG` or `NLS_NCHAR`. Moreover,
  /// `OCI_UTF16ID` is allowed to be set as charset and ncharset.
  /// On the other hand, `OCI_UTF16` mode is deprecated with this function. 
  /// Applications can achieve the same effects by setting 
  /// both charset and ncharset as `OCI_UTF16ID`.
  ///
  /// # Parameters
  /// - envhpp:
  ///   A pointer to an environment handle whose encoding setting is specified by mode.
  ///   The setting is inherited by statement handles derived from `envhpp`.
  /// - mode:
  ///   Specifies initialization of the mode.
  /// - ctxp:
  ///   Specifies the user-defined context for the memory callback routines.
  /// - malocfp:
  ///   Specifies the user-defined memory allocation function. If mode is `OCI_THREADED`, this
  ///   memory allocation routine must be thread-safe.
  /// - ralocfp:
  ///   Specifies the user-defined memory reallocation function. If the mode is `OCI_THREADED`,
  ///   this memory allocation routine must be thread-safe.
  /// - mfreefp:
  ///   Specifies the user-defined memory free function. If the mode is `OCI_THREADED`,
  ///   this memory free routine must be thread-safe.
  /// - xtramemsz:
  ///   Specifies the amount of user memory to be allocated for the duration of the environment.
  /// - usrmempp:
  ///   Returns a pointer to the user memory of size `xtramemsz` allocated by the call for the user.
  /// - charset:
  ///   The client-side character set for the current environment handle. If it is 0, the `NLS_LANG`
  ///   setting is used. `OCI_UTF16ID` is a valid setting; it is used by the metadata and the `CHAR` data.
  /// - ncharset:
  ///   The client-side national character set for the current environment handle. If it is `0`,
  ///   `NLS_NCHAR` setting is used. `OCI_UTF16ID` is a valid setting; it is used by the `NCHAR` data.
  fn OCIEnvNlsCreate(envhpp: *mut *mut OCIEnv, // результат
                     mode: c_uint,
                     ctxp: *mut c_void,
                     malocfp: Option<types::MallocFn>,
                     ralocfp: Option<types::ReallocFn>,
                     mfreefp: Option<types::FreeFn>,
                     xtramemsz: usize,
                     usrmempp: *mut *mut c_void,
                     charset: c_ushort,
                     ncharset: c_ushort) -> c_int;
  /// Detaches the process from the shared memory subsystem and releases the shared memory.
  fn OCITerminate(mode: c_uint) -> c_int;

  /// Returns a pointer to an allocated and initialized handle.
  ///
  /// # Parameters
  /// - parenth:
  ///   An environment handle.
  /// - hndlpp:
  ///   Returns a handle.
  /// - htype:
  ///   Specifies the type of handle to be allocated. The allowed handles are described in Table 2-1.
  /// - xtramem_sz:
  ///   Specifies an amount of user memory to be allocated.
  /// - usrmempp:
  ///   Returns a pointer to the user memory of size xtramem_sz allocated by the call for the user.
  fn OCIHandleAlloc(parenth: *const c_void,
                    hndlpp: *mut *mut c_void, // результат
                    htype: c_uint,
                    xtramem_sz: usize,
                    usrmempp:  *mut *mut c_void // результат
                   ) -> c_int;
  /// This call explicitly deallocates a handle.
  ///
  /// # Comments
  /// This call frees up storage associated with a handle, corresponding to the type specified in the type parameter.
  ///
  /// This call returns either `OCI_SUCCESS`, `OCI_INVALID_HANDLE`, or `OCI_ERROR`.
  ///
  /// All handles may be explicitly deallocated. The OCI deallocates a child handle if the parent is deallocated.
  ///
  /// When a statement handle is freed, the cursor associated with the statement handle is closed, but the actual
  /// cursor closing may be deferred to the next round-trip to the server. If the application must close the cursor
  /// immediately, you can make a server round-trip call, such as `OCIServerVersion()` or `OCIPing()`, after the
  /// `OCIHandleFree()` call.
  ///
  /// # Parameters
  /// - hndlp:
  ///   A handle allocated by `OCIHandleAlloc()`.
  /// - htype:
  ///   Specifies the type of storage to be freed. The handles are described in Table 2-1.
  fn OCIHandleFree(hndlp: *mut c_void,
                   htype: c_uint) -> c_int;

  /// Returns a descriptor of a parameter specified by position in the describe handle or statement handle.
  ///
  /// # Comments
  /// This call returns a descriptor of a parameter specified by position in the describe handle or statement handle.
  /// Parameter descriptors are always allocated internally by the OCI library. They can be freed using `OCIDescriptorFree()`.
  /// For example, if you fetch the same column metadata for every execution of a statement, then the program leaks memory
  /// unless you explicitly free the parameter descriptor between each call to `OCIParamGet()`.
  ///
  /// # Parameters
  /// - hndlp:
  ///   A statement handle or describe handle. The `OCIParamGet()` function returns a parameter descriptor for this handle.
  /// - htype:
  ///   The type of the handle passed in the hndlp parameter.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - parmdpp:
  ///   A descriptor of the parameter at the position given in the `pos` parameter, of handle type `OCI_DTYPE_PARAM`.
  /// - pos:
  ///   Position number in the statement handle or describe handle. A parameter descriptor is returned for this position.
  fn OCIParamGet(hndlp: *const c_void,
                 htype: c_uint,
                 errhp: *mut OCIError,
                 parmdpp:  *mut *mut c_void,
                 pos: c_uint) -> c_int;
  /// Sets a complex object retrieval (COR) descriptor into a COR handle.
  ///
  /// # Comments
  /// The COR handle must have been previously allocated using `OCIHandleAlloc()`, and the descriptor must have
  /// been previously allocated using `OCIDescriptorAlloc()`. Attributes of the descriptor are set using `OCIAttrSet()`.
  ///
  /// # Parameters
  /// - hndlp:
  ///   Handle pointer.
  /// - htype:
  ///   Handle type.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - dscp:
  ///   Complex object retrieval descriptor pointer.
  /// - dtyp:
  ///   Descriptor type. The descriptor type for a COR descriptor is `OCI_DTYPE_COMPLEXOBJECTCOMP`.
  /// - pos:
  ///   Position number.
  fn OCIParamSet(hndlp: *mut c_void,
                 htype: c_uint,
                 errhp: *mut OCIError,
                 dscp: *const c_void,
                 dtyp: c_uint,
                 pos: c_uint);

  /// Creates an access path to a data source for OCI operations.
  ///
  /// # Comments
  /// This call is used to create an association between an OCI application and a particular server.
  /// 
  /// This call assumes that OCIConnectionPoolCreate() has been called, giving poolName, when connection
  /// pooling is in effect.
  /// 
  /// This call initializes a server context handle, which must have been previously allocated with a call
  /// to `OCIHandleAlloc()`. The server context handle initialized by this call can be associated with a
  /// service context through a call to `OCIAttrSet()`. After that association has been made, OCI operations
  /// can be performed against the server.
  /// 
  /// If an application is operating against multiple servers, multiple server context handles can be maintained.
  /// OCI operations are performed against whichever server context is currently associated with the service context.
  /// 
  /// When `OCIServerAttach()` is successfully completed, an Oracle Database shadow process is started.
  /// `OCISessionEnd()` and `OCIServerDetach()` should be called to clean up the Oracle Database shadow process.
  /// Otherwise, the shadow processes accumulate and cause the Linux or UNIX system to run out of processes.
  /// If the database is restarted and there are not enough processes, the database may not start up.
  ///
  /// # Parameters
  /// - srvhp:
  ///   An uninitialized server handle, which is initialized by this call. Passing in an initialized server handle causes an error.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - dblink:
  ///   Specifies the database server to use. This parameter points to a character string that specifies a connect string
  ///   or a service point. If the connect string is `NULL`, then this call attaches to the default host. The string itself
  ///   could be in UTF-16 encoding mode or not, depending on the mode or the setting in application's environment handle.
  ///   The length of dblink is specified in `dblink_len`. The dblink pointer may be freed by the caller on return.
  ///
  ///   The name of the connection pool to connect to when `mode = OCI_CPOOL`. This must be the same as the `poolName`
  ///   parameter of the connection pool created by `OCIConnectionPoolCreate()`. Must be in the encoding specified by the 
  ///   charset parameter of a previous call to `OCIEnvNlsCreate()`.
  /// - dblink_len:
  ///   The length of the string pointed to by dblink. For a valid connect string name or alias, `dblink_len` must be nonzero.
  ///   Its value is in number of bytes.
  ///
  ///   The length of `poolName`, in number of bytes, regardless of the encoding, when `mode = OCI_CPOOL`.
  /// - mode:
  ///   Specifies the various modes of operation. Because an attached server handle can be set for any connection session
  ///   handle, the `mode` value here does not contribute to any session handle.
  fn OCIServerAttach(srvhp: *mut OCIServer,// результат
                     errhp: *mut OCIError,
                     dblink: *const c_uchar,
                     dblink_len: c_int,
                     mode: c_uint) -> c_int;
  /// Deletes an access path to a data source for OCI operations.
  ///
  /// # Comments
  /// This call deletes an access path a to data source for OCI operations. The access path was established by a
  /// call to `OCIServerAttach()`.
  ///
  /// # Parameters
  /// - srvhp:
  ///   A handle to an initialized server context, which is reset to an uninitialized state. The handle is not deallocated.
  /// - errhp:
  ///   An error handle that you can pass to `OCIErrorGet()` for diagnostic information when there is an error.
  /// - mode:
  ///   Specifies the various modes of operation. The only valid mode is `OCI_DEFAULT` for the default mode.
  fn OCIServerDetach(srvhp: *mut OCIServer,
                     errhp: *mut OCIError,
                     mode: c_uint) -> c_int;

  /// Creates a user session and begins a user session for a given server.
  fn OCISessionBegin(svchp: *mut OCISvcCtx,
                     errhp: *mut OCIError,
                     usrhp: *mut OCISession,
                     credt: c_uint,
                     mode: c_uint) -> c_int;
  /// Terminates a user session context created by `OCISessionBegin()`
  fn OCISessionEnd(svchp: *mut OCISvcCtx,
                   errhp: *mut OCIError,
                   usrhp: *mut OCISession,
                   mode: c_uint) -> c_int;

  /// Gets the value of an attribute of a handle.
  fn OCIAttrGet(trgthndlp: *const c_void,
                trghndltyp: c_uint,
                attributep: *mut c_void,
                sizep: *mut c_uint,
                attrtype: c_uint,
                errhp: *mut OCIError) -> c_int;
  /// Sets the value of an attribute of a handle or a descriptor.
  fn OCIAttrSet(trgthndlp: *mut c_void,
                trghndltyp: c_uint,
                attributep: *mut c_void,
                size: c_uint,
                attrtype: c_uint,
                errhp: *mut OCIError) -> c_int;

  /// Returns an error message in the buffer provided and an Oracle Database error code.
  fn OCIErrorGet(hndlp: *mut c_void,
                 recordno: c_uint,
                 sqlstate: *mut c_uchar,// устарел с версии 8.x
                 errcodep: *mut c_int,  // возвращаемый код ошибки
                 bufp: *mut c_uchar,    // возвращаемое сообщение об ошибке
                 bufsiz: c_uint,
                 htype: c_uint) -> c_int;
}
//-------------------------------------------------------------------------------------------------
fn check<T>(result: T, native: c_int) -> Result<T> {
  return match native {
    0 => Ok(result),
    e => Err(Error(e))
  };
}
//-------------------------------------------------------------------------------------------------
/// Автоматически освобождаемый хендл на ресурсы оракла
struct Handle<T: HandleType> {
  native: *mut T,
}
impl<T: HandleType> Handle<T> {
  fn new(env: *const OCIEnv) -> Result<Handle<T>> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
      OCIHandleAlloc(
        env as *const c_void,
        &mut handle, T::ID as c_uint,
        0, 0 as *mut *mut c_void// размер пользовательских данных и указатель на выделеное под них место
      )
    };
    return match res {
      0 => Ok(Handle { native: handle as *mut T }),
      e => Err(Error(e))
    };
  }
  fn set(&mut self, value: *mut c_void, size: c_uint, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    let res = unsafe {
      OCIAttrSet(
        self.native as *mut c_void, T::ID as c_uint,
        value, size, attrtype as c_uint,
        errhp.native
      )
    };
    return check((), res);
  }
  /// Устанавливает строковый атрибут хендлу
  fn set_str(&mut self, value: &str, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    self.set(value.as_ptr() as *mut c_void, value.len() as c_uint, attrtype, errhp)
  }
  /// Устанавливает хендл-атрибут хендлу
  fn set_handle<U: HandleType>(&mut self, value: &Handle<U>, attrtype: types::Attr, errhp: &Handle<OCIError>) -> Result<()> {
    self.set(value.native as *mut c_void, 0, attrtype, errhp)
  }
}
impl<T: HandleType> Drop for Handle<T> {
  fn drop(&mut self) {
    let res = unsafe { OCIHandleFree(self.native as *mut c_void, T::ID as c_uint) };
    check((), res).expect("OCIHandleFree");
  }
}
//-------------------------------------------------------------------------------------------------
/// Автоматически закрываемый хендл окружения оракла
struct Env {
  native: *mut OCIEnv,
  mode: types::CreateMode,
}
impl Env {
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
      0 => Ok(Env { native: handle, mode: mode }),
      e => Err(Error(e))
    };
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    Handle::new(self.native)
  }
}
impl Drop for Env {
  fn drop(&mut self) {
    let res = unsafe { OCITerminate(self.mode as c_uint) };
    check((), res).expect("OCITerminate");
  }
}
//-------------------------------------------------------------------------------------------------
pub struct Environment {
  env: Env,
  error: Handle<OCIError>,
}
impl Environment {
  pub fn new(mode: types::CreateMode) -> Result<Self> {
    let env = try!(Env::new(mode));
    let err: Handle<OCIError> = try!(env.handle());

    Ok(Environment { env: env, error: err })
  }
  pub fn connect<P: Into<ConnectParams>>(&self, params: P) -> Result<Connection> {
    let p = params.into();
    Connection::new(&self, &p.dblink, p.mode, &p.username, &p.password)
  }
  fn handle<T: HandleType>(&self) -> Result<Handle<T>> {
    self.env.handle()
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.error.native
  }
}
impl Drop for Environment {
  fn drop(&mut self) {}
}
//-------------------------------------------------------------------------------------------------
/// Хранит автоматически закрываемый хендл `OCIServer`, предоставляющий доступ к базе данных
struct Server<'env> {
  env: &'env Environment,
  handle: Handle<OCIServer>,
  mode: types::AttachMode,
}
impl<'env> Server<'env> {
  fn new<'e>(env: &'e Environment, dblink: &str, mode: types::AttachMode) -> Result<Server<'e>> {
    let server: Handle<OCIServer> = try!(env.handle());
    let res = unsafe {
      OCIServerAttach(
        server.native, env.error.native,
        dblink.as_ptr(), dblink.len() as c_int,
        mode as c_uint
      )
    };
    return match res {
      0 => Ok(Server { env: env, handle: server, mode: mode }),
      e => Err(Error(e))
    };
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.env.errorHandle()
  }
}
impl<'env> Drop for Server<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCIServerDetach(
        self.handle.native, self.errorHandle(),
        self.mode as c_uint
      )
    };
    check((), res).expect("OCIServerDetach");
  }
}
//-------------------------------------------------------------------------------------------------
/// Представляет соединение к базе данных, с определенным пользователем и паролем.
/// Соединение зависит от окружения, создавшего его, таким образом, окружение является менеджером
/// соединений. При уничтожении окружения все соединения закрываются, а незакоммиченные транзакции
/// в них откатываются.
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
        context.native,
        env.error.native,
        session.native,
        // Так как мы подключаемся и использованием имени пользователя и пароля, используем аутентификацию
        // базы данных
        types::CredentialMode::Rdbms as c_uint,
        types::AuthMode::Default as c_uint
      )
    };
    try!(check((), res));

    Ok(Connection { server: server, context: context, session: session })
  }
  fn errorHandle(&self) -> *mut OCIError {
    self.server.errorHandle()
  }
}
impl<'env> Drop for Connection<'env> {
  fn drop(&mut self) {
    let res = unsafe {
      OCISessionEnd(
        self.context.native,
        self.errorHandle(),
        self.session.native,
        types::AuthMode::Default as c_uint
      )
    };
    check((), res).expect("OCISessionEnd");
  }
}