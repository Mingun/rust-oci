//! Функции, описанные в разделе [OCI Date, Datetime, and Interval Functions][1] документации Oracle,
//! посвященном работе со временем.
//!
//! [1]: https://docs.oracle.com/database/122/LNOCI/oci-date-datetime-and-interval-functions.htm

use std::os::raw::{c_int, c_void};

use Result;
use ffi::Handle;

use super::DescriptorType;
use super::{OCIError, OCISession, OCINumber};
use super::super::types::Descriptor;

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
// По странной прихоти разработчиков оракла на разных системах имя библиотеки разное
#[cfg_attr(windows, link(name = "oci"))]
#[cfg_attr(not(windows), link(name = "clntsh"))]
extern "C" {
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
  ///   The OCI error handle. If there is an error, it is recorded in err, and this function returns OCI_ERROR. Obtain diagnostic information by calling OCIErrorGet().
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
}
