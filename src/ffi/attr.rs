//! Модуль, содержащий описания атрибутов, которые могут иметься у различных хендлов и дескрипторов.
//! В описании последняя строка означает тип данных для чтения/записи. Если до черты или после нее
//! типа нет, то значит данный атрибут нельзя читать или писать соответсвенно.

/// Атрибуты, которые можно получить или установить хендлу окружения.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Env {
  /// This attribute sets the value of `OCI_DURATION_DEFAULT` for allocation durations for the
  /// application associated with the environment handle.
  ///
  /// `OCIDuration */OCIDuration`
  AllocDuration = 37,
  /// The login name (DN) to use when connecting to the LDAP server.
  ///
  /// `oratext **/oratext *`
  BindDN = 155,
  /// When this attribute is set to `TRUE`, during `OCICacheFlush()` the objects that belong to the
  /// same table are flushed, which can considerably improve performance. An attribute value of `TRUE`
  /// should only be used when the order in which the objects are flushed is not important. When the
  /// attribute value is set to `TRUE`, it is not guaranteed that the order in which the objects are
  /// marked dirty is preserved.
  ///
  /// `boolean */boolean`
  CacheArrayFlush = 64,
  /// Sets the maximum size (high watermark) for the client-side object cache as a percentage of the
  /// optimal size. Usually you can set the value at 10%, the default, of the optimal size,
  /// `CacheOptSize`. Setting this attribute to 0 results in a value of 10 being used.
  /// The object cache uses the maximum and optimal values for freeing unused memory in the object cache.
  ///
  /// `ub4 */ub4`
  CacheMaxSize = 35,
  /// Sets the optimal size for the client-side object cache in bytes. The default value is 8
  /// megabytes (MB). Setting this attribute to 0 results in a value of 8 MB being used.
  ///
  /// `ub4 */ub4`
  CacheOptSize = 34,
  /// Local (client-side) character set ID. Users can update this setting only after creating the
  /// environment handle but before calling any other OCI functions. This restriction ensures the
  /// consistency among data and metadata in the same environment handle. When character set ID is
  /// UTF-16, an attempt to get this attribute is invalid.
  ///
  /// `ub2 */`
  EnvCharsetID = 31,
  /// Local (client-side) national character set ID. Users can update this setting only after
  /// creating the environment handle but before calling any other OCI functions. This restriction
  /// ensures the consistency among data and metadata in the same environment handle. When character
  /// set ID is UTF-16, an attempt to get this attribute is invalid.
  ///
  /// `ub2 */`
  EnvNCharsetID = 262,
  /// The name of the language used for the database sessions created from the current environment
  /// handle. While getting this value, users should pass an allocated buffer, which will be filled
  /// with the language name.
  ///
  /// `oratext **/oratext *`
  EnvNlsLanguage = 424,
  /// The name of the territory used for the database sessions created from the current environment
  /// handle. While getting this value, users should pass an allocated buffer, which will be filled
  /// with the territory name.
  ///
  /// `oratext **/oratext *`
  EnvNlsTerritory = 425,
  /// Encoding method is UTF-16. The value 1 means that the environment handle is created when the
  /// encoding method is UTF-16, whereas 0 means that it is not. This attribute value can only be
  /// set by the call to `OCIEnvCreate()` and cannot be changed later.
  ///
  /// `ub1 */`
  EnvUtf16 = 209,
  /// This attribute registers an event callback function.
  ///
  /// `/OCIEventCallback`
  EvtCbk = 304,
  /// This attribute registers a context passed to an event callback.
  ///
  /// `/void *`
  EvtCtx = 305,
  /// The current size of the memory allocated from the environment handle. This may help you track
  /// where memory is being used most in an application.
  ///
  /// `ub4 */`
  HeapAlloc = 30,
  /// The authentication mode. The following are the valid values:
  /// - 0x0: No authentication; anonymous bind.
  /// - 0x1: Simple authentication; user name and password authentication.
  /// - 0x5: SSL connection with no authentication.
  /// - 0x6: SSL: only server authentication required.
  /// - 0x7: SSL: both server authentication and client authentication are required.
  /// - 0x8: Authentication method is determined at run time.
  ///
  /// `ub2 */ub2`
  LdapAuth = 158,
  /// If the authentication method is "simple authentication" (user name and password authentication),
  /// then this attribute holds the password to use when connecting to the LDAP server.
  ///
  /// `oratext **/oratext *`
  LdapCred = 156,
  /// The administrative context of the client. This is usually the root of the Oracle Database LDAP
  /// schema in the LDAP server.
  ///
  /// `oratext **/oratext *`
  LdapCtx = 159,
  /// The name of the host on which the LDAP server runs.
  ///
  /// `oratext **/oratext *`
  LdapHost = 153,
  /// The port on which the LDAP server is listening.
  ///
  /// `ub2 */ub2`
  LdapPort = 154,
  /// Returns `TRUE` if the environment was initialized in object mode.
  ///
  /// `boolean */`
  Object = 2,
  /// This attribute sets the value of `OCI_PIN_DEFAULT` for the application associated with the
  /// environment handle.
  ///
  /// For example, if `OCI_ATTR_PINOPTION` is set to `OCI_PIN_RECENT`, and `OCIObjectPin()` is called
  /// with the pin_option parameter set to `OCI_PIN_DEFAULT`, the object is pinned in `OCI_PIN_RECENT`
  /// mode.
  ///
  /// `OCIPinOpt */OCIPinOpt`
  PinOption = 36,
  /// When this attribute is set to `TRUE`, newly created objects have non-`NULL` attributes.
  ///
  /// `boolean */boolean`
  ObjectNewNotNull = 16,
  /// When this attribute is set to `TRUE`, applications receive an `ORA-08179` error when attempting
  /// to flush an object that has been modified in the server by another committed transaction.
  ///
  /// `boolean */boolean`
  ObjectDetectChange = 32,
  /// This attribute sets the value of `OCI_DURATION_DEFAULT` for pin durations for the application
  /// associated with the environment handle.
  ///
  /// `OCIDuration */OCIDuration`
  PinDuration = 38,
  /// Returns the size of the memory currently allocated from the shared pool. This attribute works
  /// on any environment handle, but the process must be initialized in shared mode to return a
  /// meaningful value. This attribute is read as follows:
  /// ```c
  /// ub4 heapsz = 0;
  /// OCIAttrGet((void *)envhp, (ub4)OCI_HTYPE_ENV,
  ///            (void *) &heapsz, (ub4 *) 0,
  ///            (ub4)OCI_ATTR_SHARED_HEAPALLOC, errhp);
  /// ```
  ///
  /// `ub4 */`
  SharedHeadAlloc = 84,
  /// If the authentication method is SSL authentication, this attribute contains the location of
  /// the client wallet.
  ///
  /// `oratext **/oratext *`
  WallLoc = 157,
}
/// Атрибуты, которые можно получить с хендла ошибки.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Err {
  /// Returns the offset (into the DML array) at which the error occurred.
  ///
  /// `ub4 *`
  DmlRowOffset = 74,
  /// This attribute is set to `TRUE` if the error in the error handle is recoverable. If the error
  /// is not recoverable, it is set to `FALSE`.
  ///
  /// `boolean *`
  ErrorIsRecoverable = 472,
}
/// Атрибуты, которые можно получить или установить хендлу контекста.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Ctx {
  /// Returns the environment context associated with the service context.
  ///
  /// `OCIEnv **`
  Env = 5,
  InstName = 392,
  /// Allows you to determine whether an application has switched to Oracle release 7 mode (for
  /// example, through an `OCISvcCtxToLda()` call). A nonzero (`TRUE`) return value indicates that
  /// the application is currently running in Oracle release 8 mode, a zero (false) return value
  /// indicates that the application is currently running in Oracle release 7 mode.
  ///
  /// `ub1 *`
  InV8Mode = 44,
  /// When read, returns the pointer to the server context attribute of the service context.
  ///
  /// When changed, sets the server context attribute of the service context.
  ///
  /// `OCIServer **/OCIServer *`
  Server = 6,
  /// When read, returns the pointer to the authentication context attribute of the service context.
  ///
  /// When changed, sets the authentication context attribute of the service context.
  ///
  /// `OCISession **/OCISession *`
  Session = 7,
  /// Used to get and set the application's callback function on the `OCISvcCtx` handle. This function,
  /// if registered on `OCISvcCtx`, is called when a statement in the statement cache belonging to
  /// this service context is purged or when the session is ended.
  ///
  /// `*OCICallbackStmtCache/*OCICallbackStmtCache`
  StmtCacheCbk = 421,
  /// The default value of the statement cache size is 20 statements, for a statement cache-enabled
  /// session. The user can increase or decrease this value by setting this attribute on the service
  /// context handle. This attribute can also be used to enable or disable statement caching for the
  /// session, pooled or nonpooled. Statement caching can be enabled by setting the attribute to a
  /// nonzero size and disabled by setting it to zero.
  ///
  /// `ub4 */ub4`
  StmtCacheSize = 176,
  /// When read, returns the pointer to the transaction context attribute of the service context.
  ///
  /// When changed, sets the transaction context attribute of the service context.
  ///
  /// `OCITrans **/OCITrans *`
  Trans = 8,
  /// Returns `OCI_ATTR_MAXLEN_COMPAT_EXTENDED` if the `init.ora` parameter `max_string_size = extended`
  /// or returns `OCI_ATTR_MAXLEN_COMPAT_STANDARD` if the `init.ora` parameter `max_string_size = standard`.
  ///
  /// `ub1 */`
  VarTypeMaxLenCompat = 489,
}
/*
/// The following attributes are used for the server handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Srv {
  OCI_ATTR_ACCESS_BANNER = ,
  OCI_ATTR_BREAK_ON_NET_TIMEOUT = ,
  OCI_ATTR_ENV = 5,
  OCI_ATTR_EXTERNAL_NAME = ,
  OCI_ATTR_FOCBK = ,
  OCI_ATTR_INTERNAL_NAME = ,
  OCI_ATTR_INSTNAME = 392,
  OCI_ATTR_IN_V8_MODE = 44,
  OCI_ATTR_NONBLOCKING_MODE = ,
  OCI_ATTR_SERVER_GROUP = ,
  OCI_ATTR_SERVER_STATUS = ,
  OCI_ATTR_TAF_ENABLED = ,
  OCI_ATTR_USER_MEMORY = ,
  // Authentication Information Handle Attributes
  OCI_ATTR_FIXUP_CALLBACK = ,
  // User Session Handle Attributes
  OCI_ATTR_ACTION = ,
  OCI_ATTR_APPCTX_ATTR = ,
  OCI_ATTR_APPCTX_LIST = ,
  OCI_ATTR_APPCTX_NAME = ,
  OCI_ATTR_APPCTX_SIZE = ,
  OCI_ATTR_APPCTX_VALUE = ,
  OCI_ATTR_AUDIT_BANNER = ,
  OCI_ATTR_CALL_TIME = ,
  OCI_ATTR_CERTIFICATE = ,
  OCI_ATTR_CLIENT_IDENTIFIER = ,
  OCI_ATTR_CLIENT_INFO = ,
  OCI_ATTR_COLLECT_CALL_TIME = ,
  OCI_ATTR_CONNECTION_CLASS = ,
  OCI_ATTR_CURRENT_SCHEMA = ,
  OCI_ATTR_DBOP = ,
  OCI_ATTR_DEFAULT_LOBPREFETCH_SIZE = ,
  OCI_ATTR_DISTINGUISHED_NAME = ,
  OCI_ATTR_DRIVER_NAME = ,
  OCI_ATTR_EDITION = ,
  OCI_ATTR_INITIAL_CLIENT_ROLES = ,
  OCI_ATTR_LTXID = ,
  OCI_ATTR_MAX_OPEN_CURSORS = ,
  OCI_ATTR_MIGSESSION = ,
  OCI_ATTR_MODULE = ,
  OCI_ATTR_ORA_DEBUG_JDWP = ,
  OCI_ATTR_PASSWORD = ,
  OCI_ATTR_PROXY_CLIENT = ,
  OCI_ATTR_PROXY_CREDENTIALS = ,
  OCI_ATTR_PURITY = ,
  OCI_ATTR_SESSION_STATE = ,
  OCI_ATTR_SHARDING_KEY = ,
  OCI_ATTR_SHARDING_KEY_B64 = ,
  OCI_ATTR_SUPER_SHARDING_KEY = ,
  OCI_ATTR_TRANS_PROFILE_FOREIGN = ,
  OCI_ATTR_TRANSACTION_IN_PROGRESS = ,
  OCI_ATTR_USERNAME = ,

  OCI_ATTR_DBDOMAIN = ,
  OCI_ATTR_DBNAME = ,
  OCI_ATTR_INSTNAME = 392,
  OCI_ATTR_INSTSTARTTIME = ,
  OCI_ATTR_SERVICENAME = ,
}
/// The following attributes are used for the administration handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Admin {
  OCI_ATTR_ADMIN_PFILE = ,
}
/// The following attributes are used for the connection pool handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CPool {
  OCI_ATTR_CONN_TIMEOUT = ,
  OCI_ATTR_CONN_NOWAIT = ,
  OCI_ATTR_CONN_BUSY_COUNT = ,
  OCI_ATTR_CONN_OPEN_COUNT = ,
  OCI_ATTR_CONN_MIN = ,
  OCI_ATTR_CONN_MAX = ,
  OCI_ATTR_CONN_INCR = ,
}
/// The following attributes are used for the session pool handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum SPool {
  OCI_ATTR_SPOOL_AUTH = ,
  OCI_ATTR_SPOOL_BUSY_COUNT = ,
  OCI_ATTR_FOCBK = ,
  OCI_ATTR_SPOOL_GETMODE = ,
  OCI_ATTR_SPOOL_INCR = ,
  OCI_ATTR_SPOOL_MAX = ,
  OCI_ATTR_SPOOL_MAX_LIFETIME_SESSION = ,
  OCI_ATTR_SPOOL_MIN = ,
  OCI_ATTR_SPOOL_OPEN_COUNT = ,
  OCI_ATTR_SPOOL_STMTCACHESIZE = ,
  OCI_ATTR_SPOOL_TIMEOUT = ,
  OCI_ATTR_SPOOL_WAIT_TIMEOUT = ,
}
/// The following attributes are used for the transaction handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Tran {
  OCI_ATTR_TRANS_NAME = ,
  OCI_ATTR_TRANS_TIMEOUT = ,
  OCI_ATTR_XID = ,
}
/// The following attributes are used for the statement handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Stmt {
  OCI_ATTR_BIND_COUNT = ,
  OCI_ATTR_CHNF_REGHANDLE = ,
  OCI_ATTR_CQ_QUERYID = ,
  OCI_ATTR_CURRENT_POSITION = ,
  OCI_ATTR_ENV = 5,
  OCI_ATTR_FETCH_ROWID = ,
  OCI_ATTR_IMPLICIT_RESULT_COUNT = ,
  OCI_ATTR_NUM_DML_ERRORS = ,
  OCI_ATTR_PARAM_COUNT = ,
  OCI_ATTR_PARSE_ERROR_OFFSET = ,
  OCI_ATTR_PREFETCH_MEMORY = ,
  OCI_ATTR_PREFETCH_ROWS = ,
  #[deprecated]
  OCI_ATTR_ROW_COUNT = ,
  OCI_ATTR_DML_ROW_COUNT_ARRAY = ,
  OCI_ATTR_ROWID = ,
  OCI_ATTR_ROWS_FETCHED = ,
  OCI_ATTR_SQLFNCODE = ,
  OCI_ATTR_STATEMENT = ,
  OCI_ATTR_STMTCACHE_CBKCTX = ,
  OCI_ATTR_STMT_IS_RETURNING = ,
  OCI_ATTR_STMT_STATE = ,
  OCI_ATTR_STMT_TYPE = ,
  OCI_ATTR_UB8_ROW_COUNT = ,
}
/// The following attributes are used for the bind handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Bind {
  OCI_ATTR_CHAR_COUNT = ,
  OCI_ATTR_CHARSET_FORM = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_MAXCHAR_SIZE = ,
  OCI_ATTR_MAXDATA_SIZE = ,
  OCI_ATTR_PDPRC = ,
  OCI_ATTR_PDSCL = ,
  OCI_ATTR_ROWS_RETURNED = ,
}
/// The following attributes are used for the define handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Define {
  OCI_ATTR_CHAR_COUNT = ,
  OCI_ATTR_CHARSET_FORM = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_LOBPREFETCH_LENGTH = ,
  OCI_ATTR_LOBPREFETCH_SIZE = ,
  OCI_ATTR_MAXCHAR_SIZE = ,
  OCI_ATTR_PDPRC = ,
  OCI_ATTR_PDSCL = ,
}
/// The following attributes are used for the describe handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Describe {
  OCI_ATTR_PARAM = ,
  OCI_ATTR_PARAM_COUNT = ,
  OCI_ATTR_SHOW_INVISIBLE_COLUMNS = ,
}
/// The following attributes are used for the parameter descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Param {
  // Всех объектов
  OCI_ATTR_OBJ_ID = ,
  OCI_ATTR_OBJ_NAME = ,
  OCI_ATTR_OBJ_SCHEMA = ,
  OCI_ATTR_PTYPE = ,
  OCI_ATTR_TIMESTAMP = ,
  // Только таблиц или представлений
  OCI_ATTR_OBJID = ,
  OCI_ATTR_NUM_COLS = ,
  OCI_ATTR_LIST_COLUMNS = ,
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_IS_TEMPORARY = ,
  OCI_ATTR_IS_TYPED = ,
  OCI_ATTR_DURATION = ,
  // Только таблиц
  OCI_ATTR_RDBA = ,
  OCI_ATTR_TABLESPACE = ,
  OCI_ATTR_CLUSTERED = ,
  OCI_ATTR_PARTITIONED = ,
  OCI_ATTR_INDEX_ONLY = ,
  // Только процедур, функций и подпрограм
  OCI_ATTR_LIST_ARGUMENTS = ,
  OCI_ATTR_IS_INVOKER_RIGHTS = ,
  // Только подпрограм
  OCI_ATTR_NAME = ,
  OCI_ATTR_OVERLOAD_ID = ,
  // Только пакетов
  OCI_ATTR_LIST_PKG_TYPES = ,
  OCI_ATTR_LIST_SUBPROGRAMS = ,
  OCI_ATTR_IS_INVOKER_RIGHTS = ,
  // Только типов
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_TYPECODE = ,
  OCI_ATTR_COLLECTION_TYPECODE = ,
  OCI_ATTR_IS_INCOMPLETE_TYPE = ,
  OCI_ATTR_IS_SYSTEM_TYPE = ,
  OCI_ATTR_IS_PREDEFINED_TYPE = ,
  OCI_ATTR_IS_TRANSIENT_TYPE = ,
  OCI_ATTR_IS_SYSTEM_GENERATED_TYPE = ,
  OCI_ATTR_HAS_NESTED_TABLE = ,
  OCI_ATTR_HAS_LOB = ,
  OCI_ATTR_HAS_FILE = ,
  OCI_ATTR_COLLECTION_ELEMENT = ,
  OCI_ATTR_NUM_TYPE_ATTRS = ,
  OCI_ATTR_LIST_TYPE_ATTRS = ,
  OCI_ATTR_NUM_TYPE_METHODS = ,
  OCI_ATTR_LIST_TYPE_METHODS = ,
  OCI_ATTR_MAP_METHOD = ,
  OCI_ATTR_ORDER_METHOD = ,
  OCI_ATTR_IS_INVOKER_RIGHTS = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_PACKAGE_NAME = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_IS_FINAL_TYPE = ,
  OCI_ATTR_IS_INSTANTIABLE_TYPE = ,
  OCI_ATTR_IS_SUBTYPE = ,
  OCI_ATTR_SUPERTYPE_SCHEMA_NAME = ,
  OCI_ATTR_SUPERTYPE_NAME = ,
  // Атрибуты Type Attribute
  OCI_ATTR_DATA_SIZE = ,
  OCI_ATTR_TYPECODE = ,
  OCI_ATTR_DATA_TYPE = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_PRECISION = ,
  OCI_ATTR_SCALE = ,
  OCI_ATTR_PACKAGE_NAME = ,
  OCI_ATTR_TYPE_NAME = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_CHARSET_FORM = 32,
  OCI_ATTR_FSPRECISION = ,
  OCI_ATTR_FSPRECISION = ,
  // Атрибуты типа метода
  OCI_ATTR_NAME = ,
  OCI_ATTR_ENCAPSULATION = ,
  OCI_ATTR_LIST_ARGUMENTS = ,
  OCI_ATTR_IS_CONSTRUCTOR = ,
  OCI_ATTR_IS_DESTRUCTOR = ,
  OCI_ATTR_IS_OPERATOR = ,
  OCI_ATTR_IS_SELFISH = ,
  OCI_ATTR_IS_MAP = ,
  OCI_ATTR_IS_ORDER = ,
  OCI_ATTR_IS_RNDS = ,
  OCI_ATTR_IS_RNPS = ,
  OCI_ATTR_IS_WNDS = ,
  OCI_ATTR_IS_WNPS = ,
  OCI_ATTR_IS_FINAL_METHOD = ,
  OCI_ATTR_IS_INSTANTIABLE_METHOD = ,
  OCI_ATTR_IS_OVERRIDING_METHOD = ,
  // Атрибуты коллекций
  OCI_ATTR_DATA_SIZE = ,
  OCI_ATTR_TYPECODE = ,
  OCI_ATTR_DATA_TYPE = ,
  OCI_ATTR_NUM_ELEMS = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_PRECISION = ,
  OCI_ATTR_SCALE = ,
  OCI_ATTR_PACKAGE_NAME = ,
  OCI_ATTR_TYPE_NAME = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_CHARSET_FORM = 32,
  // Атрибуты синонимов
  OCI_ATTR_OBJID = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_LINK = ,
  // Атрибуты последовательностей
  OCI_ATTR_OBJID = ,
  OCI_ATTR_MIN = ,
  OCI_ATTR_MAX = ,
  OCI_ATTR_INCR = ,
  OCI_ATTR_CACHE = ,
  OCI_ATTR_ORDER = ,
  OCI_ATTR_HW_MARK = ,
  // Атрибуты колонок таблиц или представлений
  OCI_ATTR_CHAR_USED = ,
  OCI_ATTR_CHAR_SIZE = ,
  OCI_ATTR_COLLATION_ID = ,
  OCI_ATTR_COLUMN_PROPERTIES = ,
  OCI_ATTR_INVISIBLE_COL = ,
  OCI_ATTR_DATA_SIZE = ,
  OCI_ATTR_DATA_TYPE = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_PRECISION = ,
  OCI_ATTR_SCALE = ,
  OCI_ATTR_IS_NULL = ,
  OCI_ATTR_TYPE_NAME = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_CHARSET_FORM = 32,
  // Атрибуты аргументов и результата
  OCI_ATTR_NAMEв = ,
  OCI_ATTR_POSITION = ,
  OCI_ATTR_TYPECODE = ,
  OCI_ATTR_DATA_TYPE = ,
  OCI_ATTR_DATA_SIZE = ,
  OCI_ATTR_PRECISION = ,
  OCI_ATTR_SCALE = ,
  OCI_ATTR_LEVEL = ,
  OCI_ATTR_HAS_DEFAULT = ,
  OCI_ATTR_LIST_ARGUMENTS = ,
  OCI_ATTR_IOMODE = ,
  OCI_ATTR_RADIX = ,
  OCI_ATTR_IS_NULL = ,
  OCI_ATTR_TYPE_NAME = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_SUB_NAME = ,
  OCI_ATTR_LINK = ,
  OCI_ATTR_REF_TDO = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_CHARSET_FORM = 32,
  // Аргументы списков
  OCI_LTYPE_COLUMN = ,в
  OCI_LTYPE_ARG_PROC = ,
  OCI_LTYPE_ARG_FUNC = ,
  OCI_LTYPE_SUBPRG = ,
  OCI_LTYPE_TYPE_ATTR = ,
  OCI_LTYPE_TYPE_METHOD = ,
  OCI_LTYPE_TYPE_ARG_PROC = ,
  OCI_LTYPE_TYPE_ARG_FUNC = ,
  OCI_LTYPE_SCH_OBJ = ,
  OCI_LTYPE_DB_SCH = ,
  // Атрибуты схемы
  OCI_ATTR_LIST_OBJECTS = ,
  // Атрибуты базы данных
  OCI_ATTR_VERSION = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_NCHARSET_ID = ,
  OCI_ATTR_LIST_SCHEMAS = ,
  OCI_ATTR_MAX_PROC_LEN = ,
  OCI_ATTR_MAX_COLUMN_LEN = ,
  OCI_ATTR_CURSOR_COMMIT_BEHAVIOR = ,
  OCI_ATTR_MAX_CATALOG_NAMELEN = ,
  OCI_ATTR_CATALOG_LOCATION = ,
  OCI_ATTR_SAVEPOINT_SUPPORT = ,
  OCI_ATTR_NOWAIT_SUPPORT = ,
  OCI_ATTR_AUTOCOMMIT_DDL = ,
  OCI_ATTR_LOCKING_MODE = ,
  // Атрибуты правил
  OCI_ATTR_CONDITION = ,
  OCI_ATTR_EVAL_CONTEXT_OWNER = ,
  OCI_ATTR_EVAL_CONTEXT_NAME = ,
  OCI_ATTR_COMMENT = ,
  OCI_ATTR_LIST_ACTION_CONTEXT = ,
  // Атрибуты наборов правил
  OCI_ATTR_EVAL_CONTEXT_OWNER = ,
  OCI_ATTR_EVAL_CONTEXT_NAME = ,
  OCI_ATTR_COMMENT = ,
  OCI_ATTR_LIST_RULES = ,
  // Атрибуты контекста вычисления
  OCI_ATTR_EVALUATION_FUNCTION = ,
  OCI_ATTR_COMMENT = ,
  OCI_ATTR_LIST_TABLE_ALIASES = ,
  OCI_ATTR_LIST_VARIABLE_TYPES = ,
  // Атрибуты псевдонимов таблиц
  OCI_ATTR_NAME = ,
  OCI_ATTR_TABLE_NAME = ,
  // Атрибуты типов переменных
  OCI_ATTR_NAME = ,
  OCI_ATTR_TYPE = ,
  OCI_ATTR_VAR_VALUE_FUNCTION = ,
  OCI_ATTR_VAR_METHOD_FUNCTION = ,
  // Атрибуты пар имя-значение
  OCI_ATTR_NAME = ,
  OCI_ATTR_VALUE = ,
}
/// The following attributes are used for the shard instance descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Shard {
  OCI_ATTR_INSTNAME = 392,
  OCI_ATTR_SHARD_HAS_WRITABLECHUNK = ,
}
/// The following attributes are used for the parameter descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Lob {
  OCI_ATTR_LOBEMPTY = ,
  OCI_ATTR_LOB_REMOTE = ,
}
/// The following attributes are used for the complex object retrieval handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ComplexObjectHandle {
  OCI_ATTR_COMPLEXOBJECT_LEVEL = ,
  OCI_ATTR_COMPLEXOBJECT_COLL_OUTOFLINE = ,
}
/// The following attributes are used for the complex object retrieval descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ComplexObjectDescriptor {
  OCI_ATTR_COMPLEXOBJECTCOMP_TYPE = ,
  OCI_ATTR_COMPLEXOBJECTCOMP_TYPE_LEVEL = ,
}
/// The following attributes are properties of the `OCIAQEnqOptions` descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OCIAQEnqOptions {
  OCI_ATTR_MSG_DELIVERY_MODE = ,
  OCI_ATTR_RELATIVE_MSGID = ,
  OCI_ATTR_SEQUENCE_DEVIATION = ,
  OCI_ATTR_TRANSFORMATION = ,
  OCI_ATTR_VISIBILITY = ,
}
/// The following attributes are properties of the `OCIAQDeqOptions` descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OCIAQDeqOptions {
  OCI_ATTR_CONSUMER_NAME = ,
  OCI_ATTR_CORRELATION = ,
  OCI_ATTR_DEQ_MODE = ,
  OCI_ATTR_DEQ_MSGID = ,
  OCI_ATTR_DEQCOND = ,
  OCI_ATTR_MSG_DELIVERY_MODE = ,
  OCI_ATTR_NAVIGATION = ,
  OCI_ATTR_TRANSFORMATION = ,
  OCI_ATTR_VISIBILITY = ,
  OCI_ATTR_WAIT = ,
}
/// The following attributes are properties of the `OCIAQMsgProperties` descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OCIAQMsgProperties {
  OCI_ATTR_ATTEMPTS = ,
  OCI_ATTR_CORRELATION = ,
  OCI_ATTR_DELAY = ,
  OCI_ATTR_ENQ_TIME = ,
  OCI_ATTR_EXCEPTION_QUEUE = ,
  OCI_ATTR_EXPIRATION = ,
  OCI_ATTR_MSG_DELIVERY_MODE = ,
  OCI_ATTR_MSG_STATE = ,
  OCI_ATTR_ORIGINAL_MSGID = ,
  OCI_ATTR_PRIORITY = ,
  OCI_ATTR_RECIPIENT_LIST = ,
  OCI_ATTR_SENDER_ID = ,
  OCI_ATTR_TRANSACTION_NO = ,
}
/// The following attributes are properties of the `OCIAQAgent` descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OCIAQAgent {
  OCI_ATTR_AGENT_ADDRESS = ,
  OCI_ATTR_AGENT_NAME = ,
  OCI_ATTR_AGENT_PROTOCOL = ,
}
/// The following attributes are properties of the `OCIServerDNs` descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OCIServerDNs {
  OCI_ATTR_DN_COUNT = ,
  OCI_ATTR_SERVER_DN = ,
}
/// The following attributes are used for the subscription handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Subscribtion {
  OCI_ATTR_SERVER_DNS = ,
  OCI_ATTR_SUBSCR_CALLBACK = ,
  OCI_ATTR_SUBSCR_CQ_QOSFLAGS = ,
  OCI_ATTR_SUBSCR_CTX = ,
  OCI_ATTR_SUBSCR_HOSTADDR = ,
  OCI_ATTR_SUBSCR_IPADDR = ,
  OCI_ATTR_SUBSCR_NAME = ,
  OCI_ATTR_SUBSCR_NAMESPACE = ,
  OCI_ATTR_SUBSCR_NTFN_GROUPING_CLASS = ,
  OCI_ATTR_SUBSCR_NTFN_GROUPING_REPEAT_COUNT = ,
  OCI_ATTR_SUBSCR_NTFN_GROUPING_START_TIME = ,
  OCI_ATTR_SUBSCR_NTFN_GROUPING_TYPE = ,
  OCI_ATTR_SUBSCR_NTFN_GROUPING_VALUE = ,
  OCI_ATTR_SUBSCR_PAYLOAD = ,
  OCI_ATTR_SUBSCR_PORTNO = ,
  OCI_ATTR_SUBSCR_QOSFLAGS = ,
  OCI_ATTR_SUBSCR_RECPT = ,
  OCI_ATTR_SUBSCR_RECPTPRES = ,
  OCI_ATTR_SUBSCR_RECPTPROTO = ,
  OCI_ATTR_SUBSCR_TIMEOUT = ,
}
/// The following attributes are used for continuous query notification.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Chnf {
  OCI_ATTR_CHNF_CHANGELAG = ,
  OCI_ATTR_CHNF_OPERATIONS = ,
  OCI_ATTR_CHNF_ROWIDS = ,
  OCI_ATTR_CHNF_TABLENAMES = ,
}
/// The following attributes are used for the continuous query notification descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Chdes {
  OCI_ATTR_CHDES_DBNAME = ,
  OCI_ATTR_CHDES_NFTYPE = ,
  OCI_ATTR_CHDES_ROW_OPFLAGS = ,
  OCI_ATTR_CHDES_ROW_ROWID = ,
  OCI_ATTR_CHDES_TABLE_CHANGES = ,
  OCI_ATTR_CHDES_TABLE_NAME = ,
  OCI_ATTR_CHDES_TABLE_OPFLAGS = ,
  OCI_ATTR_CHDES_TABLE_ROW_CHANGES = ,
}
/// The following are attributes of the descriptor `OCI_DTYPE_AQNFY`.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Notify {
  OCI_ATTR_AQ_NTFN_GROUPING_COUNT = ,
  OCI_ATTR_AQ_NTFN_GROUPING_MSGID_ARRAY = ,
  OCI_ATTR_CONSUMER_NAME = ,
  OCI_ATTR_MSG_PROP = ,
  OCI_ATTR_NFY_FLAGS = ,
  OCI_ATTR_NFY_MSGID = ,
  OCI_ATTR_QUEUE_NAME = ,
}
/// This section describes `OCI_DTYPE_CQDES` attributes. See Oracle Database Development Guide for
/// more information about the `OCI_DTYPE_CQDES` continuous query notification descriptor.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum InvalidatedQuery {
  OCI_ATTR_CQDES_OPERATION = ,
  OCI_ATTR_CQDES_QUERYID = ,
  OCI_ATTR_CQDES_TABLE_CHANGES = ,
  = ,
}
/// The following attributes are used for the direct path context handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DirectPathCtx {
  OCI_ATTR_BUF_SIZE = ,
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_DATEFORMAT = ,
  OCI_ATTR_DIRPATH_DCACHE_DISABLE = ,
  OCI_ATTR_DIRPATH_DCACHE_HITS = ,
  OCI_ATTR_DIRPATH_DCACHE_MISSES = ,
  OCI_ATTR_DIRPATH_DCACHE_NUM = ,
  OCI_ATTR_DIRPATH_DCACHE_SIZE = ,
  OCI_ATTR_DIRPATH_FLAGS = ,
  OCI_ATTR_DIRPATH_INDEX_MAINT_METHOD = ,
  OCI_ATTR_DIRPATH_MODE = ,
  OCI_ATTR_DIRPATH_NO_INDEX_ERRORS = ,
  OCI_ATTR_DIRPATH_NOLOG = ,
  OCI_ATTR_DIRPATH_OBJ_CONSTR = ,
  OCI_ATTR_DIRPATH_PARALLEL = ,
  OCI_ATTR_DIRPATH_PGA_LIM = ,
  OCI_ATTR_DIRPATH_REJECT_ROWS_REPCH = ,
  OCI_ATTR_DIRPATH_SKIPINDEX_METHOD = ,
  OCI_ATTR_DIRPATH_SPILL_PASSES = ,
  OCI_ATTR_LIST_COLUMNS = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_NUM_COLS = ,
  OCI_ATTR_NUM_ROWS = ,
  OCI_ATTR_SCHEMA_NAME = ,
  OCI_ATTR_SUB_NAME = ,
}
/// For further explanations of these attributes, see "Direct Path Function Context and Attributes".
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DirectPathFnCtx {
  OCI_ATTR_DIRPATH_EXPR_TYPE = ,
  OCI_ATTR_LIST_COLUMNS = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_NUM_COLS = ,
  OCI_ATTR_NUM_ROWS = ,
}
/// The following attributes are used for the direct path function column array handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DirectPathFnColArr {
  OCI_ATTR_COL_COUNT = ,
  OCI_ATTR_NUM_COLS = ,
  OCI_ATTR_NUM_ROWS = ,
  OCI_ATTR_ROW_COUNT = ,
}
/// The following attributes are used for the direct path stream handle.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DirectPathStream {
  OCI_ATTR_BUF_ADDR = ,
  OCI_ATTR_BUF_SIZE = ,
  OCI_ATTR_ROW_COUNT = ,
  OCI_ATTR_STREAM_OFFSET = ,
}
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum DirectPathColParam {
  OCI_ATTR_CHARSET_ID = 31,
  OCI_ATTR_DATA_SIZE = ,
  OCI_ATTR_DATA_TYPE = ,
  OCI_ATTR_DATEFORMAT = ,
  OCI_ATTR_DIRPATH_OID = ,
  OCI_ATTR_DIRPATH_SID = ,
  OCI_ATTR_NAME = ,
  OCI_ATTR_PRECISION = ,
  OCI_ATTR_SCALE = ,
}
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Process {
  OCI_ATTR_MEMPOOL_APPNAME = ,
  OCI_ATTR_MEMPOOL_HOMENAME = ,
  OCI_ATTR_MEMPOOL_INSTNAME = ,
  OCI_ATTR_MEMPOOL_SIZE = ,
  OCI_ATTR_PROC_MODE = ,
}
/// The event callback obtains the attributes of an event using OCIAttrGet() with the following attributes.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Event {
  OCI_ATTR_DBDOMAIN = ,
  OCI_ATTR_DBNAME = ,
  OCI_ATTR_EVENTTYPE = ,
  OCI_ATTR_HA_SOURCE = ,
  OCI_ATTR_HA_SRVFIRST = ,
  OCI_ATTR_HA_SRVNEXT = ,
  OCI_ATTR_HA_STATUS = ,
  OCI_ATTR_HA_TIMESTAMP = ,
  OCI_ATTR_HOSTNAME = ,
  OCI_ATTR_INSTNAME = 392,
  OCI_ATTR_INSTSTARTTIME = ,
  OCI_ATTR_SERVICENAME = ,
}*/