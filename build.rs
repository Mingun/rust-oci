fn main() {
  let oracle_home = std::env::var("ORACLE_HOME").expect("Environment variable ORACLE_HOME not set");
  if cfg!(windows) {
  	println!("cargo:rustc-link-search={}/oci/lib/msvc", oracle_home);
  } else {
  	println!("cargo:rustc-link-search={}/lib", oracle_home);
  }
}