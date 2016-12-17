fn main() {
  let oracle_home = std::env::var("ORACLE_HOME").expect("Environment variable ORACLE_HOME not set");
  println!("cargo:rustc-link-search={}/lib", oracle_home);
}