
use std::os::raw::c_void;

pub type MallocFn  = extern "C" fn(ctxp: *mut c_void, size: usize) -> *mut c_void;
pub type ReallocFn = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void, newsize: usize) -> *mut c_void;
pub type FreeFn    = extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void);

pub type OCICallbackLobArrayRead  = extern "C" fn(ctxp: *mut c_void,
                                                  array_iter: u32,
                                                  bufp: *const c_void,
                                                  lenp: u64,
                                                  piecep: u8,
                                                  changed_bufpp: *mut *mut c_void,
                                                  changed_lenp: *mut u64) -> i32;
pub type OCICallbackLobArrayWrite = extern "C" fn(ctxp: *mut c_void,
                                                  array_iter: u32,
                                                  bufp: *mut c_void,
                                                  lenp: *mut u64,
                                                  piecep: *mut u8,
                                                  changed_bufpp: *mut *mut c_void,
                                                  changed_lenp: *mut u64) -> i32;

/// Specifies the type of credentials to use for establishing the user session
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Виды дескрипторов, которые можно создать функцией `OCIDescriptorAlloc`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Attr {
  Server = 6,
  Session = 7,
  /// Количество строк, извлеченных последним последним вызовом `OCIStmtFetch2` (для `select` выражений)
  /// или количество затронутых строк (для `update`, `insert` и `delete` выражений).
  RowCount = 9,
  /// Атрибут на хендле выражения, показывает количество колонок, извлекаемых `select` выражением
  ParamCount = 18,
  Username = 22,
  Password = 23,
  /// Тип выражения (выборка, обновление и т.п.)
  StmtType = 24,
  /// Количество строк, извлеченных в последний вызов `OCIStmtFetch2` или `OCIExecute`.
  RowFetched = 197,
  /// Количество строк, извлеченных последним последним вызовом `OCIStmtFetch2` (для `select` выражений)
  /// или количество затронутых строк (для `update`, `insert` и `delete` выражений). Значение данного атрибута
  /// представлено в виде `u64` числа, а не `u32`, как `RowCount`, но он появился только с версии 12.1.
  RowCount2 = 457,


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
  // SchemaName    = 9,
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
/// Режим кеширования подготавливаемых запросов к базе данных
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
  /// [1]: http://docs.oracle.com/database/122/LNOCI/more-oci-advanced-topics.htm#LNOCI73008
  ImplResultsCLient = 0x0400,
}
impl Default for CachingMode {
  fn default() -> Self { CachingMode::Default }
}
/// Коды ошибок, которые могут вернуть функции оракла (не путать с кодами ошибок оракла `ORA-xxxxx`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// Определяет способ связывания выходных данных для выражения.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
  /// Define noncontiguous addresses of data. The `valuep` parameter must be of the type `OCIIOV *`.
  IOV          = 1 << 9,
}
impl Default for DefineMode {
  fn default() -> Self { DefineMode::Default }
}
/// Определяет способ связывания входных данных для выражения.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BindMode {
  /// This is the default mode.
  Default    = 0,
  /// When this mode is selected, the `value_sz` parameter defines the maximum size of the data that can be provided at run time.
  /// The application must be ready to provide the OCI library runtime IN data buffers at any time and any number of times. Runtime
  /// data is provided in one of the following ways:
  ///
  /// - Callbacks using a user-defined function that must be registered with a subsequent call to `OCIBindDynamic()`.
  /// - A polling mechanism using calls supplied by OCI. This mode is assumed if no callbacks are defined.
  ///
  /// When mode is set to `DataAtExec`, do not provide values for `valuep`, `indp`, `alenp`, and `rcodep` in the main call.
  /// Pass zeros (0) for `indp` and `alenp`. Provide the values through the callback function registered using `OCIBindDynamic()`.
  DataAtExec = 1 << 1,
  /// Soft bind mode. This mode increases the performance of the call. If this is the first bind or some input value like `dty`
  /// or `value_sz` is changed from the previous bind, this mode is ignored. An error is returned if the statement is not executed.
  /// Unexpected behavior results if the bind handle passed is not valid.
  Soft       = 1 << 6,
  /// Bind noncontiguous addresses of data. The `valuep` parameter must be of the type `OCIIOV *`. This mode is intended to be
  /// used for scatter or gather binding, which allows multiple buffers to be bound or defined to a position, for example column
  /// `A` for the first 10 rows in one buffer, next 5 rows in one buffer, and the remaining 25 rows in another buffer. That
  /// eliminates the need to allocate and copy all of them into one big buffer while doing the array execute operation.
  IOV        = 1 << 9,
}
impl Default for BindMode {
  fn default() -> Self { BindMode::Default }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NumberFlag {
  Unsigned = 0,
  Signed = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(i16)]
pub enum OCIInd {
  NotNull = 0,
  Null = -1,
  BadNull = -2,
  NotNullable = -3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CallbackResult {
  /// Продолжить выполнение функции, вызвавшей функцию обратного вызова
  Continue = -24200,
  /// Завершить выполнение функции, вызвавшей функцию обратного вызова
  Done     = -24201,
}