//! Содержит структуры и типажи, предназначенные для унифицированного представления индексаторов.
use std::fmt;

use stmt::query::RowSet;

/// Типаж, позволяющий указать типы, которые можно использовать для индексации набора полей, полученных из базы данных,
/// для извлечения данных. Наиболее типичное применение -- использование индекса или имени колонки для извлечения данных.
/// Благодаря типажу для этого можно использовать одну и ту же функцию [`get()`][get].
///
/// [get]: ../struct.Row.html#method.get
pub trait RowIndex {
  /// Превращает объект в индекс, по которому можно извлечь данные, или в `None`, если нет индекса, соответствующего
  /// данному объекту. В этом случае при получении данных из столбца метод [`get()`][get] вернет ошибку [`InvalidColumn`][err].
  ///
  /// [get]: ../struct.Row.html#method.get
  /// [err]: ../../error/enum.Error.html#variant.InvalidColumn
  fn idx(&self, rs: &RowSet) -> Option<usize>;
}

impl RowIndex for usize {
  fn idx(&self, rs: &RowSet) -> Option<usize> {
    if *self >= rs.columns().len() {
      return None;
    }
    Some(*self)
  }
}
impl<'a> RowIndex for &'a str {
  fn idx(&self, rs: &RowSet) -> Option<usize> {
    rs.columns().iter().position(|x| x.name == *self)
  }
}

/// Обобщенный индекс связываемых параметров. Позволяет связывать параметры как по позиции,
/// так и по имени, используя один и тот же вызов [`bind`][1], перегруженный по принимаемым аргументам.
///
/// [1]: ../struct.Statement.html#method.bind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindIndex<'a> {
  /// Связывание осуществляется по имени переменной.
  Name(&'a str),
  /// Связывание осуществляется по позиции переменной.
  Index(usize)
}
impl<'a> fmt::Display for BindIndex<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      BindIndex::Name(name) => write!(f, "{}", name),
      BindIndex::Index(pos) => write!(f, "{}", pos),
    }
  }
}

impl<'a> From<usize> for BindIndex<'a> {
  fn from(t: usize) -> Self {
    BindIndex::Index(t)
  }
}
impl<'a> From<&'a str> for BindIndex<'a> {
  fn from(t: &'a str) -> Self {
    BindIndex::Name(t)
  }
}