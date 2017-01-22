//! Тесты извлечения и привязки столбцов/параметров с большими символьными/бинарными данными.
//! Таблица:
//! ```sql
//! create table type_lob (
//!   id number(2) not null primary key,-- номер теста
//!
//!   col0 clob,
//!   col1 nclob,
//!
//!   col2 blob,
//!   col3 bfile
//! );
//! ```

extern crate oci;

use std::io::Read;

use oci::Environment;
use oci::types::CreateMode;
use oci::lob::{Blob, Clob, BFile};
mod utils;

#[test]
fn null_extract() {
  let env = Environment::new(CreateMode::Threaded).expect("Can't init ORACLE environment in THREADED mode");
  let conn = utils::connect(&env);
  let mut stmt = conn.prepare("select * from type_lob where id = 0").expect("Can't prepare query");

  let rs = stmt.query().expect("Can't execute query");
  let row = (&rs).next().unwrap().unwrap();

  assert_eq!(None, row.get::<Clob, usize>(1).expect("Can't get CLOB"));
  assert_eq!(None, row.get::<Clob, usize>(2).expect("Can't get NCLOB"));
  assert_eq!(None, row.get::<Blob, usize>(3).expect("Can't get BLOB"));
  assert_eq!(None, row.get::<BFile,usize>(4).expect("Can't get BFILE"));
}
macro_rules! extract_test {
  ($Type:tt, $testID:expr, $column:expr, $first_part:expr, $second_part:expr) => (
    let env = Environment::new(CreateMode::Threaded).expect("Can't init ORACLE environment in THREADED mode");
    let conn = utils::connect(&env);
    let mut stmt = conn.prepare(&format!("select * from type_lob where id = {}", $testID)).expect("Can't prepare query");

    let rs = stmt.query().expect("Can't execute query");
    let row = rs.next().expect("Can't fetch").expect("Nothing fetch");

    let first : Option<$Type> = row.get($column).expect("First get failed");
    let second: Option<$Type> = row.get($column).expect("Second get failed");

    assert!(first.is_some());
    assert!(second.is_some());

    let mut f = first.expect("First value is NULL");
    let     s = second.expect("Second value is NULL");
    assert_eq!(f, s);

    direct_read(&mut f, $first_part);
    reader_read(f.new_reader().expect("Can't get reader"), $first_part, $second_part);
  );
}

/// Повторные чтения напрямую из объекта должны давать один и тот же результат
fn direct_read<R: Read>(val: &mut R, expected: &[u8]) {
  let mut buf1 = [0u8; 5];
  let mut buf2 = [0u8; 5];

  assert_eq!(5, val.read(&mut buf1).expect("Can't read data 1"));
  assert_eq!(expected, &buf1);
  assert_eq!(5, val.read(&mut buf2).expect("Can't read data 2"));
  assert_eq!(expected, &buf2);
}
/// Повторные чтения из читателя должны дать продолжающийся результат
fn reader_read<R: Read>(mut r: R, first_expected: &[u8], second_expected: &[u8]) {
  let mut buf1 = [0u8; 5];
  let mut buf2 = [0u8; 5];
  let mut buf3 = [0u8; 5];

  assert_eq!(5, r.read(&mut buf1).expect("Can't read data from reader 1"));
  assert_eq!(first_expected, &buf1);
  assert_eq!(5, r.read(&mut buf2).expect("Can't read data from reader 2"));
  assert_eq!(second_expected, &buf2);

  assert_eq!(0, r.read(&mut buf3).expect("Can't read data from reader 3"));
  assert_eq!(&[0,0,0,0,0], &buf3);
}
#[test]
fn clob_extract() {
  extract_test!(Clob, 1, 1, b"01234", b"56789");
}
#[test]
fn nclob_extract() {
  extract_test!(Clob, 1, 2, b"01234", b"56789");
}
#[test]
fn blob_extract() {
  extract_test!(Blob, 1, 3, &[0,1,2,3,4], &[5,6,7,8,9]);
}
#[test]
fn bfile_extract() {
  extract_test!(BFile, 1, 4, &[0,1,2,3,4], &[5,6,7,8,9]);
}

#[test]
#[ignore]
fn clob_extract_unicode() {
  extract_test!(Clob, 2, 1, b"01234", b"56789");
}
#[test]
#[ignore]
fn nclob_extract_unicode() {
  extract_test!(Clob, 2, 2, b"01234", b"56789");
}