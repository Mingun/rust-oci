//! Реализация конвертации в типы базы данных и обратно бинарных данных

use {Connection, Result};
use error::Error;

use types::{FromDB, Type};

impl FromDB for Vec<u8> {
  fn from_db(ty: Type, raw: &[u8], _: &Connection) -> Result<Self> {
    match ty {
      //Type::VBI |
      Type::BIN |// Тип колонки RAW
      Type::LBI => Ok(raw.into()),
      t => Err(Error::Conversion(t)),
    }
  }
}
/// Так как невозможно реализовать типаж обобщенно для любого размера массива, приходится реализовать
/// его вручную только для некоторых размеров.
///
/// В реализации в массив записывается количество данных, равное минимуму от длин полученного из базы
/// массива и массива, для которого реализуется типаж.
macro_rules! array {
  ($size:tt) => (
    impl FromDB for [u8; $size] {
      fn from_db(ty: Type, raw: &[u8], _: &Connection) -> Result<Self> {
        match ty {
          //Type::VBI |
          Type::BIN |   // Тип колонки RAW
          Type::LBI => {// Тип колонки LONG RAW
            let mut res = [0u8; $size];
            if $size < raw.len() {
              /// Записываем столько данных, сколько можем вместить
              res.copy_from_slice(&raw[0..$size])
            } else {
              /// Записываем столько данных, сколько получили
              res[0..raw.len()].copy_from_slice(raw)
            }
            Ok(res)
          },
          t => Err(Error::Conversion(t)),
        }
      }
    }
  );
}
array!(1);
array!(2);
array!(3);
array!(4);
array!(5);
array!(6);
array!(7);
array!(8);
array!(9);
array!(10);
array!(11);
array!(12);
array!(13);
array!(14);
array!(15);
array!(16);
array!(17);
array!(18);
array!(19);
array!(20);
array!(21);
array!(22);
array!(23);
array!(24);
array!(25);
array!(26);
array!(27);
array!(28);
array!(29);
array!(30);
array!(31);
array!(32);