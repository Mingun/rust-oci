//! Тесты извлечения и привязки текстовых столбцов/параметров.
//! Таблица:
//! ```sql
//! create table type_text (
//!   id number(2) not null primary key,-- номер теста
//!
//!   -- Устаревший тип столбца
//!   col0 long,
//!
//!   -- Минимальный размер
//!   col1 varchar2(1 byte),
//!   col2 varchar2(1 char),
//!
//!   -- Максимальный размер при стандартных настройках
//!   col3 varchar2(4000 byte),
//!   col4 varchar2(4000 char),
//!
//!   -- Минимальный размер
//!   col5 nvarchar2(1),
//!   -- Максимальный размер при стандартных настройках
//!   -- Размер задается в символах, а в ограничениях проверяются байты. Максимум равен 2000 (32767) байт,
//!   -- для UTF-16 это 1000 (16383) символов, для UTF-8 - ~666 (10922) символов
//!   col6 nvarchar2(1000),
//!
//!   -- Минимальный размер
//!   col7 char(1 byte),
//!   col8 char(1 char),
//!   -- Максимальный размер
//!   col9  char(2000 byte),
//!   col10 char(2000 char),
//!
//!   -- Минимальный размер
//!   col11 nchar(1),
//!   -- Максимальный размер при стандартных настройках
//!   -- Размер задается в символах, а в ограничениях проверяются байты. Максимум равен 2000 байт,
//!   -- для UTF-16 это 1000 символов, для UTF-8 - 2000/3 ~ 666 символов
//!   col12 nchar(1000)
//! );
//! ```

extern crate oci;

use oci::Environment;
use oci::types::CreateMode;
mod utils;

#[test]
fn null_extract() {
  let env = Environment::new(CreateMode::Threaded).expect("Can't init ORACLE environment in THREADED mode");
  let conn = utils::connect(&env);
  let mut stmt = conn.prepare("select * from type_text where id = 0").expect("Can't prepare query");

  let rs = stmt.query().expect("Can't execute query");
  let row = (&rs).next().unwrap().unwrap();

  for i in 1..13 {
    assert_eq!(None, row.get::<String, usize>(i).unwrap());
  }
}
fn extract_test(column: usize, expected: String) {
  let env = Environment::new(CreateMode::Threaded).expect("Can't init ORACLE environment in THREADED mode");
  let conn = utils::connect(&env);
  let mut stmt = conn.prepare("select * from type_text where id = 1").expect("Can't prepare query");

  let rs = stmt.query().expect("Can't execute query");
  let row = (&rs).next().unwrap().unwrap();

  let first  = row.get::<String, usize>(column).unwrap();
  let second = row.get::<String, usize>(column).unwrap();

  assert_eq!(Some(expected.clone()), first);
  assert_eq!(Some(expected), second);
}
#[test]
#[ignore]// Пока не работает
fn small_extract_long() {
  extract_test(1, "*".repeat(10));
}
//----VARCHAR2-------------------------------------------------------------------------------------
#[test]
fn small_extract_varchar2_1byte() {
  extract_test(2, "1".to_owned());
}
#[test]
fn small_extract_varchar2_1char() {
  extract_test(3, "2".to_owned());
}
#[test]
fn small_extract_varchar2_10byte() {
  extract_test(4, "*".repeat(10));
}
#[test]
fn small_extract_varchar2_10char() {
  extract_test(5, "*".repeat(10));
}
//----NVARCHAR2------------------------------------------------------------------------------------
#[test]
fn small_extract_nvarchar2_1() {
  extract_test(6, "3".to_owned());
}
#[test]
fn small_extract_nvarchar2_10() {
  extract_test(7, "*".repeat(10));
}
//----CHAR-----------------------------------------------------------------------------------------
#[test]
fn small_extract_char_1byte() {
  extract_test(8, "4".to_owned());
}
#[test]
fn small_extract_char_1char() {
  extract_test(9, "5".to_owned());
}
#[test]
fn small_extract_char_10byte() {
  // Тип char имеет фиксированный размер и всегда дополняется пробелами до него
  let filler = " ".repeat(2000-10);
  extract_test(10, "*".repeat(10) + &filler);
}
#[test]
fn small_extract_char_10char() {
  // Тип char имеет фиксированный размер и всегда дополняется пробелами до него
  let filler = " ".repeat(2000-10);
  extract_test(11, "*".repeat(10) + &filler);
}
//----NCHAR----------------------------------------------------------------------------------------
#[test]
fn small_extract_nchar_1() {
  extract_test(12, "6".to_owned());
}
#[test]
fn small_extract_nchar_10() {
  // Тип char имеет фиксированный размер и всегда дополняется пробелами до него
  let filler = " ".repeat(1000-10);
  extract_test(13, "*".repeat(10) + &filler);
}