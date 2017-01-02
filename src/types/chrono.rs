//! Поддержка столбцов с датой и временем из ящика `chrono`.
extern crate chrono;

use self::chrono::{Duration, NaiveDate, NaiveTime, NaiveDateTime};

use {Connection, Result};
use error::Error;
use types::{FromDB, Type};

use ffi::native::time::{get_date, get_time, Timestamp};
use ffi::native::time::{get_day_second, IntervalDS};

impl FromDB for NaiveDate {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP => {
        let t: &Timestamp = unsafe { conn.as_descriptor(raw) };
        let (yyyy, MM, dd) = try!(get_date(&conn.session, conn.error(), t));

        Ok(NaiveDate::from_ymd(yyyy as i32, MM as u32, dd as u32))
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
impl FromDB for NaiveTime {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP => {
        let t: &Timestamp = unsafe { conn.as_descriptor(raw) };
        let (hh, mm, ss, ns) = try!(get_time(&conn.session, conn.error(), t));

        Ok(NaiveTime::from_hms_nano(hh as u32, mm as u32, ss as u32, ns as u32))
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
impl FromDB for NaiveDateTime {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    let date = try!(NaiveDate::from_db(ty, raw, conn));
    let time = try!(NaiveTime::from_db(ty, raw, conn));

    Ok(NaiveDateTime::new(date, time))
  }
}

impl FromDB for Duration {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::INTERVAL_DS => {
        from_ds(ty, raw, conn)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
fn from_ds(ty: Type, raw: &[u8], conn: &Connection) -> Result<Duration> {
  let i: &IntervalDS = unsafe { conn.as_descriptor(raw) };
  let dur = try!(get_day_second(&conn.session, conn.error(), i));

  let dd = Duration::days(dur[0] as i64);
  let hh = Duration::hours(dur[1] as i64);
  let mm = Duration::minutes(dur[2] as i64);
  let ss = Duration::seconds(dur[3] as i64);
  let ns = Duration::nanoseconds(dur[4] as i64);
  Ok(dd + hh + mm + ss + ns)
}