//! Перечисляемые типы данных, используемые при работе с библиотекой

use std::u32;

/// Возможные типы данных базы данных
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// Виды выражений, которые могут быть у него после его подготовки.
/// Вид выражения влияет на то, с какими параметрыми вызывать функцию `OCIExecute()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(u16)]
pub enum StatementType {
  /// Unknown statement
  UNKNOWN = 0,
  /// Select statement
  SELECT  = 1,
  /// Update statement
  UPDATE  = 2,
  /// delete statement
  DELETE  = 3,
  /// Insert Statement
  INSERT  = 4,
  /// create statement
  CREATE  = 5,
  /// drop statement
  DROP    = 6,
  /// alter statement
  ALTER   = 7,
  /// begin ... (pl/sql statement)
  BEGIN   = 8,
  /// declare .. (pl/sql statement)
  DECLARE = 9,
  /// corresponds to kpu call
  CALL    = 10,
}
/// Виды кодировок, поддерживаемых базой данных.
///
/// В документации нигде не перечислены соответствия имени кодировки ее числовому значению, поэтому они получены
/// следующим SQL-скриптом:
/// ```sql
/// select value as name, nls_charset_id(value) as val
///   from v$nls_valid_values
///  where parameter = 'CHARACTERSET'
/// order by nls_charset_id(value)
/// ```
/// http://www.mydul.net/charsets.html
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Charset {
  /// Использовать настройки из переменных окружения `NLS_LANG` (для типов `CHAR`, `VARCHAR2` и `CLOB`)
  /// и `NLS_NCHAR` (для типов `NCHAR`, `NVARCHAR2` и `NCLOB`).
  ///
  /// Данная настройка является настройкой по умолчанию для базы данных и библиотека возвращает ее в реализации
  /// метода `default()`.
  Default        =    0,
  /// ASCII 7-bit American
  US7ASCII       =    1,
  /// IBM-PC Code Page 437 8-bit American
  US8PC437       =    4,
  /// IBM-PC Code Page 850 8-bit West European
  WE8PC850       =   10,
  /// IBM-PC Code Page 858 8-bit West European
  WE8PC858       =   28,
  /// ISO 8859-1 West European
  WE8ISO8859P1   =   31,
  /// ISO 8859-2 East European
  EE8ISO8859P2   =   32,
  /// ISO 8859-3 South European
  SE8ISO8859P3   =   33,
  /// ISO 8859-4 North and North-East European
  NEE8ISO8859P4  =   34,
  /// ISO 8859-5 Latin/Cyrillic
  CL8ISO8859P5   =   35,
  /// ISO 8859-6 Latin/Arabic
  AR8ISO8859P6   =   36,
  /// ISO 8859-7 Latin/Greek
  EL8ISO8859P7   =   37,
  /// ISO 8859-8 Latin/Hebrew
  IW8ISO8859P8   =   38,
  /// ISO 8859-9 West European & Turkish
  WE8ISO8859P9   =   39,
  /// ISO 8859-10 North European
  NE8ISO8859P10  =   40,
  /// Thai Industrial Standard 620-2533 - ASCII 8-bit
  TH8TISASCII    =   41,
  /// MS Windows Code Page 1258 8-bit Vietnamese
  VN8MSWIN1258   =   45,
  /// ISO 8859-1 West European
  WE8ISO8859P15  =   46,
  /// ISO 8859-13 Baltic
  BLT8ISO8859P13 =   47,
  /// ISO 8859-14 Celtic
  CEL8ISO8859P14 =   48,
  /// KOI8 Ukrainian Cyrillic
  CL8KOI8U       =   51,
  /// ISO 8859-9 Azerbaijani
  AZ8ISO8859P9E  =   52,
  /// IBM-PC Code Page 852 8-bit East European
  EE8PC852       =  150,
  /// IBM-PC Code Page 866 8-bit Latin/Cyrillic
  RU8PC866       =  152,
  /// IBM-PC Code Page 857 8-bit Turkish
  TR8PC857       =  156,
  /// MS Windows Code Page 1250 8-bit East European
  EE8MSWIN1250   =  170,
  /// MS Windows Code Page 1251 8-bit Latin/Cyrillic
  CL8MSWIN1251   =  171,
  /// MS Windows Code Page 923 8-bit Estonian
  ET8MSWIN923    =  172,
  /// MS Windows Code Page 1253 8-bit Latin/Greek
  EL8MSWIN1253   =  174,
  /// MS Windows Code Page 1255 8-bit Latin/Hebrew
  IW8MSWIN1255   =  175,
  /// MS Windows Code Page 921 8-bit Lithuanian
  LT8MSWIN921    =  176,
  /// MS Windows Code Page 1254 8-bit Turkish
  TR8MSWIN1254   =  177,
  /// MS Windows Code Page 1252 8-bit West European
  WE8MSWIN1252   =  178,
  /// MS Windows Code Page 1257 8-bit Baltic
  BLT8MSWIN1257  =  179,
  /// Latvian Standard LVS8-92(1) Windows/Unix 8-bit Baltic
  BLT8CP921      =  191,
  /// RELCOM Internet Standard 8-bit Latin/Cyrillic
  CL8KOI8R       =  196,
  /// IBM-PC Code Page 775 8-bit Baltic
  BLT8PC775      =  197,
  /// IBM-PC Code Page 737 8-bit Greek/Latin
  EL8PC737       =  382,
  /// ASMO Extended 708 8-bit Latin/Arabic
  AR8ASMO8X      =  500,
  /// Arabic MS-DOS 720 Server 8-bit Latin/Arabic
  AR8ADOS720     =  558,
  /// MS Windows Code Page 1256 8-Bit Latin/Arabic
  AR8MSWIN1256   =  560,
  /// EUC 24-bit Japanese
  JA16EUC        =  830,
  /// Shift-JIS 16-bit Japanese
  JA16SJIS       =  832,
  /// Same as `JA16EUC` except for the way that the wave dash and the tilde are mapped to and from Unicode
  JA16EUCTILDE   =  837,
  /// Same as `JA16SJIS` except for the way that the wave dash and the tilde are mapped to and from Unicode
  JA16SJISTILDE  =  838,
  /// KSC5601 16-bit Korean
  KO16KSC5601    =  840,
  /// MS Windows Code Page 949 Korean
  KO16MSWIN949   =  846,
  /// CGB2312-80 16-bit Simplified Chinese
  ZHS16CGB231280 =  850,
  /// GBK 16-bit Simplified Chinese
  ZHS16GBK       =  852,
  /// GB18030 32-bit Simplified Chinese
  ZHS32GB18030   =  854,
  /// EUC 32-bit Traditional Chinese
  ZHT32EUC       =  860,
  /// BIG5 16-bit Traditional Chinese
  ZHT16BIG5      =  865,
  /// MS Windows Code Page 950 Traditional Chinese
  ZHT16MSWIN950  =  867,
  /// MS Windows Code Page 950 with Hong Kong Supplementary Character Set HKSCS-2001 (character set conversion to and from Unicode is based on Unicode 3.0)
  ZHT16HKSCS     =  868,
  /// Unicode 3.0 UTF-8 Universal character set, CESU-8 compliant
  UTF8           =  871,
  /// Unicode 7.0 UTF-8 Universal character set
  AL32UTF8       =  873,
  /// Unicode 7.0 UTF-16 Universal character set
  AL16UTF16      = 2000,
}
impl Default for Charset {
  fn default() -> Self {
    Charset::Default
  }
}