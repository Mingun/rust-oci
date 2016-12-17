fn main() {
  let oracle_home = std::env::var("ORACLE_HOME").expect("Cannot get ORACLE_HOME environment variable");
  println!("cargo:rustc-link-search={}/lib", oracle_home);
}