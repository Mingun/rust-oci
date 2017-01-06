
use oci::{Environment, Connection};
use oci::params::{ConnectParams, Credentials};
use oci::types::AuthMode;

pub fn connect<'e>(env: &'e Environment) -> Connection<'e> {
  let params = ConnectParams {
    dblink: "".into(),
    attach_mode: Default::default(),
    credentials: Credentials::Ext,
    auth_mode: AuthMode::SysDba,
  };
  env.connect(params).expect("Can't connect to ORACLE database")
}