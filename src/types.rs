//! Перечисляемые типы данных, используемые при работе с библиотекой

use std::u32;

/// Возможные типы данных базы данных
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(deprecated)]// Позволяем deprecated внутри перечисления из-за https://github.com/rust-lang/rust/issues/38832
#[repr(u16)]
pub enum Type {
  /// (ORANET TYPE) character string. У колонок с типами `varchar2/nvarchar2`.
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
  #[deprecated(note="Not recommented to use by Oracle, use LOB instead")]
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
  /// binary data(DTYBIN). У колонок с типом `raw`.
  BIN  = 23,
  /// long binary. У колонок с типом `long raw`.
  LBI  = 24,
  /// unsigned integer
  UIN  = 68,
  /// Display sign leading separate
  SLS  = 91,
  /// Longer longs (char)
  LVC  = 94,
  /// Longer long binary
  LVB  = 95,
  /// Ansi fixed char. У колонок с типами `char/nchar`.
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

/// Режим, в котором создавать окружение при вызове `OCIEnvNlsCreate()`.
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
  Default = 0,
  /// Use connection pooling.
  CPool   = 1 << 9,
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
/// Диалект Oracle-а, используемый для разбора SQL-кода запросов. Рекомендуется всегда использовать нативный для сервера
/// диалект, он является диалектом по умолчанию при выполнении [`prepare`][1] без параметров.
///
/// [1]: ../struct.Connection.html#method.prepare
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Syntax {
  /// Синтаксис зависит от версии сервера базы данных.
  Native = 1,
  /// V7 ORACLE parsing syntax.
  V7 = 2,
  //V8 = 3,
  /// Specifies the statement to be translated according to the SQL translation profile set in the session.
  Foreign = u32::MAX as isize,
}
impl Default for Syntax {
  fn default() -> Self { Syntax::Native }
}