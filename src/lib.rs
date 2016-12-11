#![feature(associated_consts)]

pub mod ffi;

pub struct Error(std::os::raw::c_int);
type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    
  }
}
