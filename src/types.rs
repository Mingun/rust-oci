use std::str;
use Result;
use error::Error;

/// Возможные типы данных базы данных
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
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

/// Преобразует тип базы данных в тип Rust, для которого реализован данный типаж.
pub trait FromDB {
  fn from_db(ty: Type, raw: &[u8]) -> Result<&Self>;
}

macro_rules! simple_from {
  ($ty:ty, $($types:ident),+) => (
    impl FromDB for $ty {
      fn from_db(ty: Type, raw: &[u8]) -> Result<&Self> {
        match ty {
          $(Type::$types)|+ => Ok(unsafe { &*(raw.as_ptr() as *const $ty) }),
          t => Err(Error::Conversion(t)),
        }
      }
    }
  )
}
simple_from!(f32, BFLOAT);
simple_from!(f64, BDOUBLE);

impl FromDB for str {
  fn from_db(ty: Type, raw: &[u8]) -> Result<&Self> {
    match ty {
      Type::CHR => str::from_utf8(raw).map_err(|_| Error::Conversion(Type::CHR)),
      t => Err(Error::Conversion(t)),
    }
  }
}