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
  ($Type:tt, $testID:expr, $column:expr, $read_len:expr, $first_part:expr, $second_part:expr) => (
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

    direct_read(&mut f, $read_len, $first_part);
    reader_read(f.new_reader().expect("Can't get reader"), $read_len, $first_part, $second_part);
  );
}

/// Проверяет, что повторные чтения из объекта дают один и тот же результат.
///
/// # Параметры
///
/// - `r`:
///   Проверяемый читатель
/// - `read_len`:
///   размер буфера для чтения. Для `[N]CLOB`-ов требуется указывать размер с 4-х кратным
///   резервированием, если нужно, чтобы Oracle прочитал `X` байт, то необходим буфер с размером
///   не менее `4*X` байт.
/// - `expected`:
///   Буфер с ожидаемым результатом первого и второго чтения.
fn direct_read<R: Read>(val: &mut R, read_len: usize, expected: &[u8]) {
  // Из-за необъяснимого поведения Oracle необходимо 4-х кратное резервирование для чтения
  // любых символов из [N]CLOB. К сожалению, ответа, похоже, никто не знает, как это побороть.
  // http://stackoverflow.com/questions/42051848/oracle-doesnt-use-the-output-buffer-completely-in-ocilobread2-it-is-possibly-f
  let mut buf1 = [0u8; 5*4];
  let mut buf2 = [0u8; 5*4];
  let len = expected.len();

  assert_eq!(len, val.read(&mut buf1[0..read_len]).expect("Can't read data 1"));
  assert_eq!(expected, &buf1[0..len]);
  assert_eq!(len, val.read(&mut buf2[0..read_len]).expect("Can't read data 2"));
  assert_eq!(expected, &buf2[0..len]);
}
/// Проверяет, что повторные чтения из читателя читают следующие куски данных, без пропусков и наложений.
///
/// # Параметры
///
/// - `r`:
///   Проверяемый читатель
/// - `read_len`:
///   размер буфера для чтения. Для `[N]CLOB`-ов требуется указывать размер с 4-х кратным
///   резервированием, если нужно, чтобы Oracle прочитал `X` байт, то необходим буфер с размером
///   не менее `4*X` байт.
/// - `first_expected`:
///   Буфер с ожидаемым результатом первого чтения.
/// - `second_expected`:
///   Буфер с ожидаемым результатом второго чтения.
fn reader_read<R: Read>(mut r: R, read_len: usize, first_expected: &[u8], second_expected: &[u8]) {
  // Из-за необъяснимого поведения Oracle необходимо 4-х кратное резервирование для чтения
  // любых символов из [N]CLOB. К сожалению, ответа, похоже, никто не знает, как это побороть.
  // http://stackoverflow.com/questions/42051848/oracle-doesnt-use-the-output-buffer-completely-in-ocilobread2-it-is-possibly-f
  let mut buf1 = [0u8; 5*4];
  let mut buf2 = [0u8; 5*4];
  let mut buf3 = [0u8; 5*4];
  let len1 = first_expected.len();
  let len2 = second_expected.len();

  assert_eq!(len1, r.read(&mut buf1[0..read_len]).expect("Can't read data from reader 1"));
  assert_eq!(first_expected, &buf1[0..len1]);
  assert_eq!(len2, r.read(&mut buf2[0..read_len]).expect("Can't read data from reader 2"));
  assert_eq!(second_expected, &buf2[0..len2]);

  assert_eq!(0, r.read(&mut buf3).expect("Can't read data from reader 3"));
  assert_eq!(&[0; 5*4], &buf3);
}
#[test]
fn clob_extract() {
  extract_test!(Clob, 1, 1, 5*4, b"01234", b"56789");
}
#[test]
fn nclob_extract() {
  extract_test!(Clob, 1, 2, 5*4, b"01234", b"56789");
}
#[test]
fn blob_extract() {
  extract_test!(Blob, 1, 3, 5, &[0,1,2,3,4], &[5,6,7,8,9]);
}
#[test]
fn bfile_extract() {
  extract_test!(BFile, 1, 4, 5, &[0,1,2,3,4], &[5,6,7,8,9]);
}

#[test]
fn clob_extract_unicode() {
  // В каждой части 3 символа в кодировке UTF-16 (внутренняя кодировка оракла):
  // первая часть суррогатной пары, вторая часть суррогатной пары, ASCII символ
  extract_test!(Clob, 2, 1, 3*4, "𐌼1".as_bytes(), "2𐌰".as_bytes());
}
#[test]
fn nclob_extract_unicode() {
  // В каждой части 3 символа в кодировке UTF-16 (внутренняя кодировка оракла):
  // первая часть суррогатной пары, вторая часть суррогатной пары, ASCII символ
  extract_test!(Clob, 2, 2, 3*4, "𐌼1".as_bytes(), "2𐌰".as_bytes());
}