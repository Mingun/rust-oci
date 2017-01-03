Биндинг Oracle Call Interface для Rust
--------------------------------------
[![Build Status](https://travis-ci.org/Mingun/rust-oci.svg?branch=master)](https://travis-ci.org/Mingun/rust-oci)

# Пример использования
```rust
extern crate oci;
use oci::Environment;
use oci::params::{ConnectParams, Credentials};
use oci::types::{AttachMode, AuthMode, CreateMode};
use oci::version::client_version;

fn main() {
  // Инициализируем клиентскую библиотеку Oracle
  let env = Environment::new(CreateMode::default()).expect("Can't create ORACLE environment");

  // Создаем параметры. Вскоре их можно будет распарсить из строки (jdbc и sql*plus версий)
  let params = ConnectParams {
    dblink: "".into(),
    attach_mode: AttachMode::default(),
    // Учетные данные, в данном случае аутентификация по паролю.
    credentials: Credentials::Rdbms { username: "username".into(), password: "password".into() },
    auth_mode: AuthMode::default(),
  };

  // Соединяемся с сервером
  let conn = env.connect(params).expect("Can't connect to ORACLE database");
  println!("Client version: {}", client_version());
  println!("Server version: {}", conn.server_version().expect("Can't get server version"));
  println!("Connection time offset: {:?}", conn.get_current_time_offset().expect("Can't get connection time offset"));

  // Готовим запрос для выполнения
  let stmt = conn.prepare("select * from user_users").expect("Can't prepare statement");

  // Выполняем! Bind-параметры пока не поддерживаются
  let rs = stmt.query().expect("Can't execute query");
  // ...продолжение следует...
}
```