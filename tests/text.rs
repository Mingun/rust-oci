//! Тесты извлечения и привязки текстовых столбцов/параметров.
//! Таблица:
//! ```sql
//! create table type_text (
//!   id number(2) not null,-- номер теста
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
mod utils;

#[test]
fn null_extract() {
  let env = Environment::new(Default::default()).unwrap();
  let conn = utils::connect(&env);
  let mut stmt = conn.prepare("select * from type_text where id = 0").expect("Can't prepare query");

  let rs = stmt.query().expect("Can't execute query");
  let row = (&rs).next().unwrap().unwrap();

  for i in 1..13 {
    assert_eq!(None, row.get::<String, usize>(i).unwrap());
  }
}