-- Удаляем пользователя для примеров в документации, если он есть
declare
  does_not_exist   exception;
  pragma EXCEPTION_INIT (does_not_exist, -1918);
begin
  execute immediate 'drop user username';
exception
  when does_not_exist then null;
end;
/
-- Пользователь для для примеров из документации, чтобы они выполнялись без ошибок
create user username identified by password;
/
grant CONNECT, RESOURCE to username;
/