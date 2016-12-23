
use std;
use std::os::raw::{c_int, c_void, c_uchar, c_ulonglong, c_uint};

pub type MallocFn  = extern "C" fn(ctxp: *mut c_void, size: usize) -> *mut c_void;
pub type ReallocFn = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void, newsize: usize) -> *mut c_void;
pub type FreeFn    = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void);

pub type OCICallbackLobArrayRead  = extern "C" fn(ctxp: *mut c_void,
                                                  array_iter: c_uint,
                                                  bufp: *const c_void,
                                                  lenp: c_ulonglong,
                                                  piecep: c_uchar,
                                                  changed_bufpp: *mut *mut c_void,
                                                  changed_lenp: *mut c_ulonglong) -> c_int;
pub type OCICallbackLobArrayWrite = extern "C" fn(ctxp: *mut c_void,
                                                  array_iter: c_uint,
                                                  bufp: *mut c_void,
                                                  lenp: *mut c_ulonglong,
                                                  piecep: *mut c_uchar,
                                                  changed_bufpp: *mut *mut c_void,
                                                  changed_lenp: *mut c_ulonglong) -> c_int;

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
impl Default for CreateMode {
  fn default() -> Self { CreateMode::Default }
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
impl Default for AttachMode {
  fn default() -> Self { AttachMode::Default }
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
impl Default for AuthMode {
  fn default() -> Self { AuthMode::Default }
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
/// Виды хендлов, которые можно выделять функцией `OCIHandleAlloc`.
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
  //AuthInfo = 9,// нельзя иметь 2 элемента с одинаковым значением
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

/// Виды дескрипторов, которые можно создать фунцией `OCIDescriptorAlloc`
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Descriptor {
  /// Specifies generation of a LOB value type locator (for a `BLOB` or `CLOB`) of C type `OCILobLocator`.
  Lob = 50,
  /// Specifies generation of snapshot descriptor of C type `OCISnapshot`
  Snapshot = 51,
  //ResultSet = 52,
  /// Specifies generation of a read-only parameter descriptor of C type `OCIParam`.
  Param = 53,
  /// Specifies generation of a `ROWID` descriptor of C type `OCIRowid`.
  RowID = 54,
  /// Specifies generation of a complex object retrieval descriptor of C type `OCIComplexObjectComp`.
  ComplexObjectComp = 55,
  /// Specifies generation of a `FILE` value type locator of C type `OCILobLocator`.
  File = 56,
  /// Specifies generation of an Advanced Queuing enqueue options descriptor of C type `OCIAQEnqOptions`.
  AQEnqOptions = 57,
  /// Specifies generation of an Advanced Queuing dequeue options descriptor of C type `OCIAQDeqOptions`.
  AQDeqOptions = 58,
  /// Specifies generation of an Advanced Queuing message properties descriptor of C type `OCIAQMsgProperties`.
  AQMsgProperties = 59,
  /// Specifies generation of an Advanced Queuing agent descriptor of C type `OCIAQAgent`.
  AQAgent = 60,
  //Locator = 61,
  /// Specifies generation of an `INTERVAL YEAR TO MONTH` descriptor of C type `OCIInterval`.
  IntervalYM = 62,
  /// Specifies generation of an `INTERVAL DAY TO SECOND` descriptor of C type `OCIInterval`.
  IntervalDS = 63,
  /// Specifies generation of an Advanced Queuing notification descriptor of C type `OCIAQNotify`.
  AQNotify = 64,
  /// Specifies generation of an ANSI DATE descriptor of C type `OCIDateTime`.
  Date = 65,
  //Time = 66,
  //TimeWithTZ = 67,
  /// Specifies generation of a TIMESTAMP descriptor of C type `OCIDateTime`.
  Timestamp = 68,
  /// Specifies generation of a `TIMESTAMP WITH TIME ZONE` descriptor of C type `OCIDateTime`.
  TimestampWithTZ = 69,
  /// Specifies generation of a `TIMESTAMP WITH LOCAL TIME ZONE` descriptor of C type `OCIDateTime`.
  TimestampWithLTZ = 70,
  /// Specifies generation of a user callback descriptor of C type `OCIUcb`.
  UCB           = 71,
  /// Specifies generation of a Distinguished Names descriptor of C type `OCIServerDNs`.
  ServerDN      = 72,
  //Signature     = 73,
  /// Specifies generation of an Advanced Queuing listen descriptor of C type `OCIAQListenOpts`.
  AQListenOptions = 75,
  /// Specifies generation of an Advanced Queuing message properties descriptor of C type `OCIAQLisMsgProps`.
  AQListenMsgProperties = 76,
  //Change         = 77,
  //TableChange    = 78,
  //RowChange      = 79,
  //QueryChange    = 80,
  //LobRegion      = 81,
  // Specifies generation of the shard key or the shard group key of C type `OCIShardingKey`.
  //ShardingKey,// с версии 12.2c, найти API данной версии на сайте оракла не удалось
}
/// Виды атрибутов, которые можно назначать хендлам
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Attr {
  Server = 6,
  Session = 7,
  // RowCount = 9,
  /// Атрибут на хендле выражения, показывает количество колонок, извлекаемых `select` выражением
  ParamCount = 18,
  Username = 22,
  Password = 23,


// Attributes common to Columns and Stored Procs
  /// maximum size of the data
  DataSize      = 1,
  /// the SQL type of the column/argument
  DataType      = 2,
  /// the display size
  DisplaySize   = 3,
  /// the name of the column/argument
  Name          = 4,
  /// precision if number type
  Precision     = 5,
  /// scale if number type
  //Scale         = 6,
  /// is it null ?
  //IsNull        = 7,
  /// name of the named data type or a package name for package private types
  TypeName      = 8,
  /// the schema name
  SchemaName    = 9,
  /// type name if package private type
  SubName       = 10,
  /// relative position of col/arg in the list of cols/args
  Position      = 11,
  /// package name of package type
  PackageName   = 12,
// complex object retrieval parameter attributes
  ComplexObjectCompType        = 50, 
  ComplexObjectCompTypeLevel   = 51,
  ComplexObjectLevel           = 52,
  ComplexObjectCollOutOfLine   = 53,

// Only Columns
  /// the display name
  DisplayName      = 100,
  /// encrypted data size
  EncryptedSize    = 101,
  /// column is encrypted ?
  ColEncrypted     = 102,
  /// is encrypted column salted ?
  ColEncryptedSalt = 103,
  /// column properties
  ColProps         = 104,
}
/// Диалект, используемый для разбора SQL-кода запросов
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Syntax {
  /// Syntax depends upon the version of the server.
  Native = 1,
  /// V7 ORACLE parsing syntax.
  V7 = 2,
  //V8 = 3,
  /// Specifies the statement to be translated according to the SQL translation profile set in the session.
  Foreign = std::u32::MAX as isize,
}
impl Default for Syntax {
  fn default() -> Self { Syntax::Native }
}
/// Режим кеширования подготавливаемых запросов к базе данных
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CachingMode {
  /// Caching is not enabled. This is the only valid setting. If the statement is not found in the cache, this mode
  /// allocates a new statement handle and prepares the statement handle for execution. If the statement is not found
  /// in the cache and one of the following circumstances applies, then the subsequent actions follow:
  /// - Only the text has been supplied: a new statement is allocated and prepared and returned. The tag `NULL`.
  ///   `OCI_SUCCESS` is returned.
  /// - Only the tag has been supplied: stmthp is `NULL`. `OCI_ERROR` is returned.
  /// - Both text and key were supplied: a new statement is allocated and prepared and returned. The tag `NULL`.
  ///   `OCI_SUCCESS_WITH_INFO` is returned, as the returned statement differs from the requested statement in that
  ///   the tag is `NULL`.
  Default = 0,
  /// In this case, if the statement is not found (a `NULL` statement handle is returned), you must take further
  /// action. If the statement is found, `OCI_SUCCESS` is returned. Otherwise, `OCI_ERROR` is returned.
  CacheSearchOnly   = 0x0010,
  /// If warnings are enabled in the session and the `PL/SQL` program is compiled with warnings, then
  /// `OCI_SUCCESS_WITH_INFO` is the return status from the execution. Use `OCIErrorGet()` to find the new error
  /// number corresponding to the warnings.
  GetPLSQLWarnings  = 0x0020,
  /// The mode should be passed as `OCI_PREP2_IMPL_RESULTS_CLIENT` when this call is made in an external procedure
  /// and implicit results need to be processed. See ["OCI Support for Implicit Results"][1] for more details.
  ///
  /// [1]: http://docs.oracle.com/database/121/LNOCI/oci10new.htm#CEGJCAJI
  ImplResultsCLient = 0x0400,
}
impl Default for CachingMode {
  fn default() -> Self { CachingMode::Default }
}
/// Коды ошибок, которые могут вырнуть функции оракла (не путать с кодами ошибок оракла `ORA-xxxxx`)
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ErrorCode {
  /// Все в порядке, проблем нет
  Success = 0,
  /// Функция выполнилась успешно, но есть диагностическая информация, которая может быть получена вызовом(ами) `OCIErrorGet`.
  SuccessWithInfo = 1,
  /// Вызов функции не вернул данных. Это ожидаемая ошибка, которая должна быть соответствующе обработана.
  NoData = 100,
  /// При выполнении функции произошла ошибка. Вызов `OCIErrorGet` вернет подробности.
  Error = -1,
  InvalidHandle = -2,
  /// Приложение должно предоставить больше данных и повторно вызвать функцию.
  NeedData = 99,
  /// Контекст сервера в неблокирующем режиме и сейчас выполняется операция, которая не может быть прервана примо сейчас.
  /// Нужно посторить вызов функции через некоторое время, чтобы получить результат.
  StiilExecuting = -3123,
  /// Передается пользовательским Callback-ом для уведомления оракла, что необходимо продолжить выполнение
  Continue = -24200,
  /// This code is returned only from a callback function. It indicates that the callback function is done with the user row callback.
  RowCallbackDone = -24201,
}
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ExecuteMode {
  /// Calling `OCIStmtExecute()` in this mode executes the statement. It also implicitly returns describe information
  /// about the select list.
  Default = 0,
  //BatchMode = 1 << 0,
  /// Used when the application knows in advance exactly how many rows it is fetching. This mode turns prefetching off for
  /// Oracle Database release 8 or later mode, and requires that defines be done before the execute call. Using this mode
  /// cancels the cursor after the desired rows are fetched and may result in reduced server-side resource usage.
  ExactFetch = 1 << 1,
  /// Required for the result set to be scrollable. The result set cannot be updated. See "Fetching Results" for more
  /// information about this mode. This mode cannot be used with any other mode.
  StmtScrollableReadonly = 1 << 3,
  /// This mode is for users who want to describe a query before execution. Calling `OCIStmtExecute()` in this mode does
  /// not execute the statement, but it does return the select-list description. To maximize performance, Oracle recommends
  /// that applications execute the statement in default mode and use the implicit describe that accompanies the execution.
  DescribeOnly = 1 << 4,
  /// When a statement is executed in this mode, the current transaction is committed after execution, if execution
  /// completes successfully.
  CommitOnSuccess = 1 << 5,
  //NonBlocking = 1 << 6,
  /// See "Batch Error Mode" for information about this mode.
  BatchErrors = 1 << 7,
  /// This mode allows the user to parse the query before execution. Executing in this mode parses the query and returns
  /// parse errors in the SQL, if any. Users must note that this involves an additional round-trip to the server. To maximize
  /// performance, Oracle recommends that the user execute the statement in the default mode, which, parses the statement as
  /// part of the bundled operation.
  ParseOnly = 1 << 8,
  //ShowDmlWarnings = 1 << 10,
  //ResultCache = 1 << 17,
  //NoResultCache = 1 << 18,
  /// This mode allows the user to get DML rowcounts per iteration. It is an error to pass this mode for statements that
  /// are not DMLs. See "Statement Handle Attributes" for more information. This mode can be used along with `BatchErrors`.
  ReturnRowCountArray = 1 << 20,
}
impl Default for ExecuteMode {
  fn default() -> Self { ExecuteMode::Default }
}
/// Определяет способ получения данных из курсора
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum FetchMode {
  /// Has the same effect as `Next`.
  Default  = 0,
  /// Gets the current row.
  Current  = 1 << 0,
  /// Gets the next row from the current position. It is the default (has the same effect as `Default`).
  /// Use for a nonscrollable statement handle.
  Next     = 1 << 1,
  /// Gets the first row in the result set.
  First    = 1 << 2,
  /// Gets the last row in the result set.
  Last     = 1 << 3,
  /// Positions the result set on the previous row from the current row in the result set. You can fetch multiple rows using
  /// this mode, from the "previous row" also.
  Prior    = 1 << 4,
  /// Fetches the row number (specified by `fetchOffset` parameter) in the result set using absolute positioning.
  Absolute = 1 << 5,
  /// Fetches the row number (specified by `fetchOffset` parameter) in the result set using relative positioning.
  Relative = 1 << 6,
}
impl Default for FetchMode {
  fn default() -> Self { FetchMode::Default }
}
/// Определяет способ связывания данных для выражения.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DefineMode {
  /// This is the default mode.
  Default      = 0,
  /// For applications requiring dynamically allocated data at the time of fetch, this mode must be used. You can define a callback
  /// using the `OCIDefineDynamic()` call. The `value_sz` parameter defines the maximum size of the data that is to be provided at
  /// run time. When the client library needs a buffer to return the fetched data, the callback is invoked to provide a runtime
  /// buffer into which a piece or all the data is returned.
  DynamicFetch = 1 << 1,
  /// Soft define mode. This mode increases the performance of the call. If this is the first define, or some input parameter such
  /// as `dty` or `value_sz` is changed from the previous define, this mode is ignored. Unexpected behavior results if an invalid
  /// define handle is passed. An error is returned if the statement is not executed.
  Soft         = 1 << 7,
  ///  Define noncontiguous addresses of data. The `valuep` parameter must be of the type `OCIIOV *`.
  IOV          = 1 << 9,
}
impl Default for DefineMode {
  fn default() -> Self { DefineMode::Default }
}
/// Возможные типы данных базы данных
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(u16)]
pub enum Type {
  /// (ORANET TYPE) character string
  CHR  = 1,
  /// (ORANET TYPE) oracle numeric
  NUM  = 2,
  /// (ORANET TYPE) integer 
  INT  = 3,
  /// (ORANET TYPE) Floating point number
  FLT  = 4,
  /// zero terminated string
  STR  = 5,
  /// NUM with preceding length byte 
  VNU  = 6,
  /// (ORANET TYPE) Packed Decimal Numeric
  PDN  = 7,
  /// long
  LNG  = 8,
  /// Variable character string
  VCS  = 9,
  /// Null/empty PCC Descriptor entry
  NON  = 10,
  /// rowid
  RID  = 11,
  /// date in oracle format
  DAT  = 12,
  /// binary in VCS format
  VBI  = 15,
  /// Native Binary float
  BFLOAT = 21,
  /// NAtive binary double
  BDOUBLE = 22,
  /// binary data(DTYBIN)
  BIN  = 23,
  /// long binary
  LBI  = 24,
  /// unsigned integer
  UIN  = 68,
  /// Display sign leading separate
  SLS  = 91,
  /// Longer longs (char)
  LVC  = 94,
  /// Longer long binary
  LVB  = 95,
  /// Ansi fixed char
  AFC  = 96,
  /// Ansi Var char
  AVC  = 97,
  /// binary float canonical
  IBFLOAT  = 100,
  /// binary double canonical
  IBDOUBLE = 101,
  /// cursor  type
  CUR  = 102,
  /// rowid descriptor
  RDD  = 104,
  /// label type
  LAB  = 105,
  /// oslabel type
  OSL  = 106,

  /// named object type
  NTY  = 108,
  /// ref type
  REF  = 110,
  /// character lob
  CLOB = 112,
  /// binary lob
  BLOB = 113,
  /// binary file lob
  BFILEE = 114,
  /// character file lob
  CFILEE = 115,
  /// result set type
  RSET = 116,
  /// named collection type (varray or nested table)
  NCO  = 122,
  /// OCIString type
  VST  = 155,
  /// OCIDate type
  ODT  = 156,

// datetimes and intervals
  /// ANSI Date
  DATE          = 184,
  /// TIME
  TIME          = 185,
  /// TIME WITH TIME ZONE
  TIME_TZ       = 186,
  /// TIMESTAMP
  TIMESTAMP     = 187,
  /// TIMESTAMP WITH TIME ZONE
  TIMESTAMP_TZ  = 188,
  /// INTERVAL YEAR TO MONTH
  INTERVAL_YM   = 189,
  /// INTERVAL DAY TO SECOND
  INTERVAL_DS   = 190,
  ///         /*  */
  TIMESTAMP_LTZ = 232,

  /// pl/sql representation of named types
  PNTY   = 241,

// some pl/sql specific types
  /// pl/sql 'record' (or %rowtype)
  REC    = 250,
  /// pl/sql 'indexed table'
  TAB    = 251,
  /// pl/sql 'boolean'
  BOL    = 252,
}