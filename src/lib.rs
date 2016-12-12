#![feature(associated_consts)]

#[derive(Debug)]
pub struct Error(std::os::raw::c_int);
type Result<T> = std::result::Result<T, Error>;

mod ffi;
pub use ffi::*;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    
  }
}
