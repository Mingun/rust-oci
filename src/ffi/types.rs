
use std::os::raw::c_void;

pub type MallocFn  = extern "C" fn(ctxp: *mut c_void, size: usize) -> *mut c_void;
pub type ReallocFn = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void, newsize: usize) -> *mut c_void;
pub type FreeFn    = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void);

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CreateMode {
  /// The default value, which is non-UTF-16 encoding.
  Default                   = 0,
  /// Uses threaded environment. Internal data structures not exposed to the user are protected from concurrent
  /// accesses by multiple threads.
  Threaded                  = 1 << 0,
  /// Uses object features.
  Object                    = 1 << 1,
  /// Uses publish-subscribe notifications.
  Events                    = 1 << 2,
  //Shared                    = 1 << 4,
  /// Suppresses the calling of the dynamic callback routine OCIEnvCallback(). The default behavior is to allow
  /// calling of OCIEnvCallback() when the environment is created.
  /// See Also:
  /// "Dynamic Callback Registrations"
  NoUcb                     = 1 << 6,
  /// No mutual exclusion (mutex) locking occurs in this mode. All OCI calls done on the environment handle,
  /// or on handles derived from the environment handle, must be serialized. `OCI_THREADED` must also be specified
  /// when `OCI_ENV_NO_MUTEX` is specified.
  EnvNoMutex                = 1 << 7,
  //SharedExt                 = 1 << 8,
  //AlwaysBlocking            = 1 << 10,
  //UseLDAP                   = 1 << 12,
  //RegLDAPOnly               = 1 << 13,
  //UTF16                     = 1 << 14,
  //AFC_PAD_ON                = 1 << 15,
  //NewLengthSemantics        = 1 << 17,
  //NoMutexStmt               = 1 << 18,
  //MutexEnvOnly              = 1 << 19,
  /// Suppresses NLS character validation; NLS character validation suppression is on by default beginning with
  /// Oracle Database 11g Release 1 (11.1). Use `OCI_ENABLE_NLS_VALIDATION` to enable NLS character validation.
  /// See Comments for more information.
  SuppressNlsValidation     = 1 << 20,
  //OCI_MUTEX_TRY                 = 1 << 21,
  /// Turns on N' substitution.
  NCharLiteralReplaceOn     = 1 << 22,
  /// Turns off N' substitution. If neither this mode nor `OCI_NCHAR_LITERAL_REPLACE_ON` is used, the substitution
  /// is determined by the environment variable `ORA_NCHAR_LITERAL_REPLACE`, which can be set to `TRUE` or `FALSE`.
  /// When it is set to TRUE, the replacement is turned on; otherwise it is turned off, the default setting in OCI.
  NCharLiteralReplaceOff    = 1 << 23,
  /// Enables NLS character validation. See Comments for more information.
  EnableNlsValidation       = 1 << 24,
}
/// Режим, в котором подключаться к cерверу базы данных при вызове `OCIServerAttach()`.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum AttachMode {
  /// For encoding, this value tells the server handle to use the setting in the environment handle.
  Default,
  /// Use connection pooling.
  CPool,
}
/// Specifies the various modes of operation
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum AuthMode {
  /// In this mode, the user session context returned can only ever be set with the server context
  /// specified in `svchp`. For encoding, the server handle uses the setting in the environment handle.
  Default = 0,
  /// In this mode, the new user session context can be set in a service handle with a different server handle.
  /// This mode establishes the user session context. To create a migratable session, the service handle must already
  /// be set with a nonmigratable user session, which becomes the "creator" session of the migratable session. That is,
  /// a migratable session must have a nonmigratable parent session.
  ///
  /// `Migrate` should not be used when the session uses connection pool underneath. The session migration and multiplexing
  /// happens transparently to the user.
  Migrate     = 1 << 0,
  /// In this mode, you are authenticated for `SYSDBA` access
  SysDba      = 1 << 1,
  /// In this mode, you are authenticated for `SYSOPER` access
  SysOper     = 1 << 2,
  /// This mode can only be used with `SysDba` or `SysOper` to authenticate for certain administration tasks
  PrelimAuth  = 1 << 3,
  //PICache     = 1 << 4,
  /// Enables statement caching with default size on the given service handle. It is optional to pass this mode
  /// if the application is going to explicitly set the size later using `OCI_ATTR_STMTCACHESIZE` on that service handle.
  StmtCache   = 1 << 6,
  //StatelessCall = 1 << 7,
  //StatelessTxn  = 1 << 8,
  //StatelessApp  = 1 << 9,
  //SysAsm      = 1 << 14,
  //SysBkp      = 1 << 16,
  //SysDgd      = 1 << 17,
  //SysKmt      = 1 << 18,
}
/// Specifies the type of credentials to use for establishing the user session
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CredentialMode {
  /// Authenticate using a database user name and password pair as credentials.
  /// The attributes `OCI_ATTR_USERNAME` and `OCI_ATTR_PASSWORD` should be set on the user session context before this call.
  Rdbms = 1 << 0,
  /// Authenticate using external credentials. No user name or password is provided.
  Ext   = 1 << 2,
  //Proxy = 1 << 3,
}
/// Виды хендлов, которые можно выделять функцией `alloc_handle`.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Handle {
  /// OCI environment handle
  /// OCIEnv
  Env = 1,
  /// OCI error handle
  /// OCIError
  Error = 2,
  /// OCI service context handle
  /// OCISvcCtx
  SvcCtx = 3,
  /// OCI statement handle
  /// OCIStmt
  Stmt = 4,
  /// OCI bind handle
  /// OCIBind
  Bind = 5,
  /// OCI define handle
  /// OCIDefine
  Define = 6,
  /// OCI describe handle
  /// OCIDescribe
  Describe = 7,
  /// OCI server handle
  /// OCIServer
  Server = 8,
  /// OCI user session handle
  /// OCISession
  Session = 9,
  /// OCI authentication information handle
  /// OCIAuthInfo
  //AuthInfo = 9,// нельзя иметь 2 элемента с одинаковым значыением
  /// OCI transaction handle
  /// OCITrans
  Trans = 10,
  /// OCI complex object retrieval (COR) handle
  /// OCIComplexObject
  ComplexObject = 11,
  //Security = 12,
  /// OCI subscription handle
  /// OCISubscription
  Subscription = 13,
  /// OCI direct path context handle
  /// OCIDirPathCtx
  DirPathCtx = 14,
  /// OCI direct path column array handle
  /// OCIDirPathColArray
  DirPathColArray = 15,
  /// OCI direct path stream handle
  /// OCIDirPathStream
  DirPathStream = 16,
  /// OCI process handle
  /// OCIProcess
  Process = 17,
  /// OCI direct path function context handle
  /// OCIDirPathFuncCtx
  DirPathFuncCtx = 18,
  //DirPathFuncColArray = 19,
  //XADSession = 20,
  //XADTable = 21,
  //XADField = 22,
  //XADGranule = 23,
  //XADRecord = 24,
  //XADIO = 25,
  /// OCI connection pool handle
  /// OCICPool
  CPool = 26,
  /// OCI session pool handle
  /// OCISPool
  SPool = 27,
  /// OCI administration handle
  /// OCIAdmin
  Admin = 28,
  //Event = 29,
}
/// Виды атрибутов, которые можно назначать хендлам
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Attr {
  Server = 6,
  Username = 22,
  Password = 23,
}