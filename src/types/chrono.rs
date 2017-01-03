//! Поддержка столбцов с датой и временем из ящика `chrono`.
extern crate chrono;

use self::chrono::{NaiveDate, NaiveTime, NaiveDateTime};  // простые конвертации
use self::chrono::{Date, DateTime, TimeZone, FixedOffset, UTC};// с учетом часовых поясов
use self::chrono::Duration;// продолжительности времени

use {Connection, Result};
use error::Error;
use types::{FromDB, Type};

use ffi::native::time::{get_date, get_time, get_time_offset, OCIDateTime, Timestamp, TimestampWithTZ, TimestampWithLTZ};
use ffi::native::time::{get_day_second, IntervalDS};

/// Вспомогательная функция для формирования даты без знаний о часовом поясе из оракловских данных
fn to_naive_date<T: OCIDateTime>(conn: &Connection, timestamp: &T) -> Result<NaiveDate> {
  let (yyyy, MM, dd) = try!(get_date(&conn.session, conn.error(), timestamp));

  Ok(NaiveDate::from_ymd(yyyy as i32, MM as u32, dd as u32))
}
/// Вспомогательная функция для формирования времени без знаний о часовом поясе из оракловских данных
fn to_naive_time<T: OCIDateTime>(conn: &Connection, timestamp: &T) -> Result<NaiveTime> {
  let (hh, mm, ss, ns) = try!(get_time(&conn.session, conn.error(), timestamp));

  Ok(NaiveTime::from_hms_nano(hh as u32, mm as u32, ss as u32, ns as u32))
}
/// Вспомогательная функция для формирования часового пояса из оракловских данных
fn to_tz<T: OCIDateTime>(conn: &Connection, timestamp: &T) -> Result<FixedOffset> {
  let (hh, mm) = try!(get_time_offset(&conn.session, conn.error(), timestamp));
  let offset = Duration::hours(hh as i64) + Duration::minutes(mm as i64);
/*FIXME: Добавить проверку на выход за границу диапазона
  if offset.num_seconds() > i32::MAX as i64
  || offset.num_seconds() < i32::MIN as i64 {
    return Err(Error::Conversion())
  }*/
  Ok(FixedOffset::east(offset.num_seconds() as i32))
}

impl FromDB for NaiveDate {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP => {// Время в некоем неизвестном часовом поясе
        let t: &Timestamp = unsafe { conn.as_descriptor(raw) };
        to_naive_date(conn, t)
      },
      Type::TIMESTAMP_LTZ => {
        // Наивное время является текущим временем данной колонки в текущем часовом поясе. Т.е. если в базе хранится время 00:00:00,
        // сама база во времени +00:00, то в часовом поясе сессии +05:00 наивное время будет 05:00:00, а сессии в +03:00 -- 03:00:00.
        let t: &TimestampWithLTZ = unsafe { conn.as_descriptor(raw) };
        to_naive_date(conn, t)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
impl FromDB for NaiveTime {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP => {// Время в некоем неизвестном часовом поясе
        let t: &Timestamp = unsafe { conn.as_descriptor(raw) };
        to_naive_time(conn, t)
      },
      Type::TIMESTAMP_LTZ => {
        // Наивное время является текущим временем данной колонки в текущем часовом поясе. Т.е. если в базе хранится время 00:00:00,
        // сама база во времени +00:00, то в часовом поясе сессии +05:00 наивное время будет 05:00:00, а сессии в +03:00 -- 03:00:00.
        let t: &TimestampWithLTZ = unsafe { conn.as_descriptor(raw) };
        to_naive_time(conn, t)
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
//-------------------------------------------------------------------------------------------------
fn to_date<T: OCIDateTime>(conn: &Connection, timestamp: &T) -> Result<Date<FixedOffset>> {
  let (yyyy, MM, dd) = try!(get_date(&conn.session, conn.error(), timestamp));
  let tz = try!(to_tz(conn, timestamp));

  Ok(tz.ymd(yyyy as i32, MM as u32, dd as u32))
}
fn to_datetime<T: OCIDateTime>(conn: &Connection, timestamp: &T) -> Result<DateTime<FixedOffset>> {
  let (yyyy, MM, dd) = try!(get_date(&conn.session, conn.error(), timestamp));
  let (hh, mm, ss, ns) = try!(get_time(&conn.session, conn.error(), timestamp));
  let tz = try!(to_tz(conn, timestamp));

  Ok(tz.ymd(yyyy as i32, MM as u32, dd as u32).and_hms_nano(hh as u32, mm as u32, ss as u32, ns as u32))
}
impl FromDB for Date<FixedOffset> {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP_TZ => {// Время в некоем часовом поясе и сам этот пояс
        let t: &TimestampWithTZ = unsafe { conn.as_descriptor(raw) };
        to_date(conn, t)
      },
      Type::TIMESTAMP_LTZ => {// Время в некоем часовом поясе и сам этот пояс
        let t: &TimestampWithLTZ = unsafe { conn.as_descriptor(raw) };
        to_date(conn, t)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
impl FromDB for DateTime<FixedOffset> {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    match ty {
      Type::TIMESTAMP_TZ => {// Время в некоем часовом поясе и сам этот пояс
        let t: &TimestampWithTZ = unsafe { conn.as_descriptor(raw) };
        to_datetime(conn, t)
      },
      Type::TIMESTAMP_LTZ => {// Время в некоем часовом поясе и сам этот пояс
        let t: &TimestampWithLTZ = unsafe { conn.as_descriptor(raw) };
        to_datetime(conn, t)
      },
      t => Err(Error::Conversion(t)),
    }
  }
}
//-------------------------------------------------------------------------------------------------
impl FromDB for Date<UTC> {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    let time = try!(Date::<FixedOffset>::from_db(ty, raw, conn));

    Ok(time.with_timezone(&UTC))
  }
}
impl FromDB for DateTime<UTC> {
  fn from_db(ty: Type, raw: &[u8], conn: &Connection) -> Result<Self> {
    let time = try!(DateTime::<FixedOffset>::from_db(ty, raw, conn));

    Ok(time.with_timezone(&UTC))
  }
}
//-------------------------------------------------------------------------------------------------
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