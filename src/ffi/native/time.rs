//! Функции, описанные в разделе [OCI Date, Datetime, and Interval Functions][1] документации Oracle,
//! посвященном работе со временем.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/oci-date-datetime-and-interval-functions.htm

use std::os::raw::{c_char, c_uchar, c_short, c_int, c_uint, c_void};

use Result;

use ffi::Handle;// Основные типобезопасные примитивы
use ffi::DescriptorType;// Типажи для безопасного моста к FFI

use ffi::native::{OCIError, OCISession, OCINumber};// FFI типы
use ffi::types::Descriptor;

/// `OCITime` - OCI TiMe portion of date
///
/// This structure should be treated as an opaque structure as the format
/// of this structure may change. Use `OCIDateGetTime/OCIDateSetTime`
/// to manipulate time portion of `OCIDate`.
#[derive(Debug)]
#[repr(C)]
pub struct OCITime {
  /// hours; range is 0 <= hours <=23
  pub hh: u8,
  /// minutes; range is 0 <= minutes <= 59
  pub mi: u8,
  /// seconds; range is 0 <= seconds <= 59
  pub ss: u8,
}
impl Default for OCITime {
  fn default() -> Self {
    OCITime { hh: 0, mi: 0, ss: 0 }
  }
}
#[derive(Debug)]
#[repr(C)]
pub struct OCIDate {
  /// gregorian year; range is -4712 <= year <= 9999
  pub yyyy: i16,
  /// month; range is 1 <= month < 12
  pub mm: u8,
  /// day; range is 1 <= day <= 31
  pub dd: u8,
  pub time: OCITime,
}
impl Default for OCIDate {
  fn default() -> Self {
    OCIDate { yyyy: 1, mm: 1, dd: 1, time: Default::default() }
  }
}

pub trait OCIDateTime : DescriptorType {}
descriptor!(OCIDateTime, Date);
//descriptor!(OCIDateTime, Time);
//descriptor!(OCIDateTime, TimeWithTZ);
descriptor!(OCIDateTime, Timestamp);
descriptor!(OCIDateTime, TimestampWithTZ);
descriptor!(OCIDateTime, TimestampWithLTZ);

pub fn get_date<T: OCIDateTime>(hndl: &Handle<OCISession>, err: &Handle<OCIError>, datetime: &T) -> Result<(i16, u8, u8)> {
  let mut yyyy: i16 = 0;
  let mut mm: u8 = 0;
  let mut dd: u8 = 0;
  let res = unsafe {
    OCIDateTimeGetDate(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      datetime as *const T as *const c_void,
      &mut yyyy,
      &mut mm,
      &mut dd,
    )
  };
  match res {
    0 => Ok((yyyy, mm, dd)),
    e => Err(err.decode(e))
  }
}

pub fn get_time<T: OCIDateTime>(hndl: &Handle<OCISession>, err: &Handle<OCIError>, datetime: &T) -> Result<(u8, u8, u8, u32)> {
  let mut hh: u8 = 0;
  let mut mm: u8 = 0;
  let mut ss: u8 = 0;
  let mut ns: u32 = 0;
  let res = unsafe {
    OCIDateTimeGetTime(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      datetime as *const T as *mut c_void,
      &mut hh,
      &mut mm,
      &mut ss,
      &mut ns,
    )
  };
  match res {
    0 => Ok((hh, mm, ss, ns)),
    e => Err(err.decode(e))
  }
}
pub fn get_time_offset<T: OCIDateTime>(hndl: &Handle<OCISession>, err: &Handle<OCIError>, datetime: &T) -> Result<(i8, i8)> {
  let mut hh: i8 = 0;
  let mut mm: i8 = 0;
  let res = unsafe {
    OCIDateTimeGetTimeZoneOffset(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      datetime as *const T as *mut c_void,
      &mut hh,
      &mut mm,
    )
  };
  match res {
    0 => Ok((hh, mm)),
    e => Err(err.decode(e))
  }
}

//-------------------------------------------------------------------------------------------------
pub trait OCIInterval : DescriptorType {}
descriptor!(OCIInterval, IntervalYM);
descriptor!(OCIInterval, IntervalDS);

/// Получает из указателя на интервал Oracle количество лет и месяцев
pub fn get_year_month(hndl: &Handle<OCISession>, err: &Handle<OCIError>, interval: &IntervalYM) -> Result<[c_int; 2]> {
  let mut time: [c_int; 2] = [0; 2];
  let res = unsafe {
    OCIIntervalGetYearMonth(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      &mut time[0],// год
      &mut time[1],// месяц
      interval as *const IntervalYM as *const c_void
    )
  };
  match res {
    0 => Ok(time),
    e => Err(err.decode(e))
  }
}
/// Получает из указателя на интервал Oracle количество дней, часов, минут, секунд и наносекунд, которое он представляет
pub fn get_day_second(hndl: &Handle<OCISession>, err: &Handle<OCIError>, interval: &IntervalDS) -> Result<[c_int; 5]> {
  let mut time: [c_int; 5] = [0; 5];
  let res = unsafe {
    OCIIntervalGetDaySecond(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      &mut time[0],// день
      &mut time[1],// час
      &mut time[2],// минута
      &mut time[3],// секунда
      &mut time[4],// миллисекунда
      interval as *const IntervalDS as *const c_void
    )
  };
  match res {
    0 => Ok(time),
    e => Err(err.decode(e))
  }
}
pub fn to_number<T: OCIInterval>(hndl: &Handle<OCISession>, err: &Handle<OCIError>, interval: &T) -> Result<OCINumber> {
  let mut num = OCINumber::default();
  let res = unsafe {
    OCIIntervalToNumber(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      interval as *const T as *mut c_void,
      &mut num as *mut OCINumber
    )
  };
  match res {
    0 => Ok(num),
    e => Err(err.decode(e))
  }
}
//-------------------------------------------------------------------------------------------------
pub fn sys_timestamp<T: OCIDateTime>(hndl: &Handle<OCISession>, err: &Handle<OCIError>, sys_date: *mut T) -> Result<()> {
  let res = unsafe {
    OCIDateTimeSysTimeStamp(
      hndl.native_mut() as *mut c_void,
      err.native_mut(),
      sys_date as *mut c_void
    )
  };
  err.check(res)
}
//-------------------------------------------------------------------------------------------------
// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
  /// Gets the date (year, month, day) portion of a datetime value.
  ///
  /// # Comments
  /// This function gets the date (year, month, day) portion of a datetime value.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or environment handle.
  ///- err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - datetime (IN):
  ///   Pointer to an OCIDateTime descriptor from which date information is retrieved.
  /// - year (OUT):
  /// - month (OUT):
  /// - day (OUT):
  ///   The retrieved year, month, and day values.
  ///
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_ERROR`, if the input type is `SQLT_TIME` or `OCI_TIME_TZ`.
  fn OCIDateTimeGetDate(hndl: *mut c_void,
                        err: *mut OCIError,
                        // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                        datetime: *const c_void/*OCIDateTime*/,
                        year: *mut c_short,
                        month: *mut c_uchar,
                        day: *mut c_uchar) -> c_int;

  /// Gets the time (hour, min, second, fractional second) of a datetime value.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or environment handle.
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - datetime (IN):
  ///   Pointer to an OCIDateTime descriptor from which time information is retrieved.
  /// - hour (OUT):
  ///   The retrieved hour value.
  /// - min (OUT):
  ///   The retrieved minute value.
  /// - sec (OUT):
  ///   The retrieved second value.
  /// - fsec (OUT):
  ///   The retrieved fractional second value.
  /// 
  /// # Comments
  /// This function gets the time portion (hour, min, second, fractional second) from a given datetime value.
  /// 
  /// This function returns an error if the given datetime does not contain time information.
  /// 
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_ERROR`, if datetime does not contain time (`SQLT_DATE`).
  fn OCIDateTimeGetTime(hndl: *mut c_void,
                        err: *mut OCIError,
                        // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                        datetime: *mut c_void/*OCIDateTime*/,
                        hour: *mut c_uchar,
                        min: *mut c_uchar,
                        sec: *mut c_uchar,
                        fsec: *mut c_uint) -> c_int;
  /// Gets the time zone (hour, minute) portion of a datetime value.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or environment handle.
  /// - err (IN/OUT)
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - datetime (IN):
  ///   Pointer to an OCIDateTime descriptor.
  /// - hour (OUT):
  ///   The retrieved time zone hour value.
  /// - min (OUT):
  ///   The retrieved time zone minute value.
  /// 
  /// # Comments
  /// This function gets the time zone hour and the time zone minute portion from a given datetime value.
  /// 
  /// This function returns an error if the given datetime does not contain time information.
  /// 
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_ERROR`, if datetime does not contain a time zone (`SQLT_DATE`, `SQLT_TIMESTAMP`).
  fn OCIDateTimeGetTimeZoneOffset(hndl: *mut c_void,
                                  err: *mut OCIError,
                                  // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                                  datetime: *const c_void/*OCIDateTime*/,
                                  hour: *mut c_char,
                                  min: *mut c_char) -> c_int;

//-------------------------------------------------------------------------------------------------
  /// Gets values of day, hour, minute, and second from an interval.
  /// 
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or the environment handle.
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns
  ///   `OCI_ERROR`. Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - dy (OUT):
  ///   Number of days.
  /// - hr (OUT):
  ///   Number of hours.
  /// - mm (OUT):
  ///   Number of minutes.
  /// - ss (OUT):
  ///   Number of seconds.
  /// - fsec (OUT):
  ///   Number of nano seconds.
  /// - interval (IN):
  ///   The input interval.
  /// 
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_INVALID_HANDLE`, if `err` is a `NULL` pointer.
  fn OCIIntervalGetDaySecond(hndl: *mut c_void,
                             err: *mut OCIError,
                             dy: *mut c_int,
                             hr: *mut c_int,
                             mm: *mut c_int,
                             ss: *mut c_int,
                             fsec: *mut c_int,
                             // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно 2 разных типа enum-а
                             interval: *const c_void/*OCIInterval*/) -> c_int;

  /// Gets year and month from an interval.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or the environment handle.
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns
  ///   `OCI_ERROR`. Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - yr (OUT):
  ///   Year value.
  /// - mnth (OUT):
  ///   Month value.
  /// - interval (IN):
  ///   The input interval.
  /// 
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_INVALID_HANDLE`, if `err` is a `NULL` pointer.
  fn OCIIntervalGetYearMonth(hndl: *mut c_void,
                             err: *mut OCIError,
                             yr: *mut c_int,
                             mnth: *mut c_int,
                             // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно 2 разных типа enum-а
                             interval: *const c_void/*OCIInterval*/) -> c_int;
  /// Converts an interval to an Oracle NUMBER.
  ///
  /// # Comments
  /// Fractional portions of the date (for instance, minutes and seconds if the unit chosen is hours)
  /// are included in the Oracle `NUMBER` produced. Excess precision is truncated.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or the environment handle.
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - interval (IN):
  ///   Interval to be converted.
  /// - number (OUT):
  ///   Oracle `NUMBER` result (in years for `YEARMONTH` interval and in days for `DAYSECOND`).
  /// 
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_INVALID_HANDLE`, if `err` is a `NULL` pointer.
  fn OCIIntervalToNumber(hndl: *mut c_void, 
                         err: *mut OCIError,
                         // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно 2 разных типа enum-а
                         interval: *mut c_void/*OCIInterval*/,
                         number: *mut OCINumber) -> c_int;
//-------------------------------------------------------------------------------------------------
  /// Gets the system current date and time as a time stamp with time zone.
  ///
  /// # Parameters
  /// - hndl (IN):
  ///   The OCI user session handle or environment handle.
  /// - err (IN/OUT):
  ///   The OCI error handle. If there is an error, it is recorded in `err`, and this function returns `OCI_ERROR`.
  ///   Obtain diagnostic information by calling `OCIErrorGet()`.
  /// - sys_date (OUT):
  ///   Pointer to the output time stamp.
  ///
  /// # Returns
  /// `OCI_SUCCESS`; or `OCI_INVALID_HANDLE`, if `err` is a `NULL` pointer.
  fn OCIDateTimeSysTimeStamp(hndl: *mut c_void,
                             err: *mut OCIError, 
                             // Мапим на void*, т.к. использовать типажи нельзя, а нам нужно несколько разных типов enum-ов
                             sys_date: *mut c_void/*OCIDateTime*/) -> c_int;
}
