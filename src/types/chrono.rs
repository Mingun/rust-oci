//! Поддержка столбцов с датой и временем из ящика `chrono`.
extern crate chrono;

use self::chrono::Duration;

use {Connection, Result};
use error::Error;
use types::{FromDB, Type};

use ffi::native::time::{get_day_second, IntervalDS};

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