extern crate oci;

use oci::Environment;
use oci::params::{ConnectParams, Credentials};
use oci::types::{AttachMode, AuthMode, CreateMode};

mod utils;

fn connect(cred: Credentials) {
  let mode = CreateMode::default();
  let env = Environment::new(mode).expect(format!("Can't create ORACLE environment with CreateMode={:?}", mode).as_str());
  let params = ConnectParams {
    dblink: "".into(),
    attach_mode: AttachMode::default(),
    credentials: cred.clone(),
    auth_mode: AuthMode::default(),
  };
  env.connect(params).expect(format!("Can't connect to ORACLE database with Credentials={:?}", cred).as_str());
}

#[test]
fn can_be_inited() {
  let mode = CreateMode::default();
  // Выполняем дважды, из-за https://community.oracle.com/thread/3779405
  { Environment::new(mode).expect(format!("Can't create ORACLE environment with CreateMode={:?}", mode).as_str()); }
  { Environment::new(mode).expect(format!("Can't create ORACLE environment with CreateMode={:?}", mode).as_str()); }
}

#[test]
fn can_connect_with_external_authentification() {
  connect(Credentials::Ext);
}
#[test]
fn can_connect_with_rdbms_authentification_with_known_user_and_password() {
  connect(Credentials::Rdbms { username: "username".into(), password: "password".into() });
}
#[test]
#[should_panic(expected = "ORA-01017: invalid username/password; logon denied")]
fn cant_connect_with_rdbms_authentification_with_known_user_and_wrong_password() {
  connect(Credentials::Rdbms { username: "username".into(), password: "wrong_password".into() });
}
#[test]
#[should_panic(expected = "ORA-01017: invalid username/password; logon denied")]
fn cant_connect_with_rdbms_authentification_with_unknown_user() {
  connect(Credentials::Rdbms { username: "non_exist_username".into(), password: "some_password".into() });
}

#[test]
fn can_prepare() {
  let env = Environment::new(CreateMode::default()).unwrap();
  let conn = utils::connect(&env);

  let mut stmt = conn.prepare("select * from dual").expect("Can't prepare SELECT expression");
  stmt.query().expect("Can't execute SELECT expression");
  // Почему-то явно некорректный SQL не приводит к возникновению ошибки при подготовке выражения
  let mut stmt = conn.prepare("select * from").expect("Can't prepare SELECT expression");
  assert!(stmt.query().is_err());

  let stmt = conn.prepare("create table test (id number)").expect("Can't prepare DDL expression");
  stmt.execute().expect("Can't execute DDL expression");
  conn.prepare("drop table test")
      .expect("Can't prepare DROP TABLE expression")
      .execute()
      .expect("Can't execute DROP TABLE expression");
  // Почему-то явно некорректный SQL не приводит к возникновению ошибки при подготовке выражения
  let stmt = conn.prepare("create table test").expect("Can't prepare invalid DDL expression");
  assert!(stmt.execute().is_err());
}
