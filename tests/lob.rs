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

use std::fmt::Debug;

use oci::Environment;
use oci::lob::{Blob, Clob, BFile};
use oci::convert::FromDB;
mod utils;

#[test]
fn null_extract() {
  let env = Environment::default();
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
  ($Type:ty, $column:expr) => (
    let env = Environment::default();
    let conn = utils::connect(&env);
    let mut stmt = conn.prepare("select * from type_lob where id = 1").expect("Can't prepare query");

    let rs = stmt.query().expect("Can't execute query");
    let row = rs.next().expect("Can't fetch").expect("Nothing fetch");

    let first : Option<$Type> = row.get($column).expect("First get failed");
    let second: Option<$Type> = row.get($column).expect("Second get failed");

    assert!(first.is_some());
    assert!(second.is_some());

    let f = first.expect("First value is NULL");
    let s = second.expect("Second value is NULL");
    assert_eq!(f, s);
  );
}
#[test]
fn clob_extract() {
  extract_test!(Clob, 1);
}
#[test]
fn nclob_extract() {
  extract_test!(Clob, 2);
}
#[test]
fn blob_extract() {
  extract_test!(Blob, 3);
}
#[test]
fn bfile_extract() {
  extract_test!(BFile, 4);
}