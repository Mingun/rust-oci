---------------------------------------------------------------------------------------------------
-- Вспомогательная функция для чистки базы. Не нужна при непрерывной интеграции, но полезна при разработке,
-- когда база не откатывается каждый раз
create or replace procedure delete_table(table_name in varchar2)
as
begin
  declare
    does_not_exist   exception;
    pragma EXCEPTION_INIT (does_not_exist, -942);
  begin
    execute immediate 'drop table '||table_name||' purge';
  exception
    when does_not_exist then null;
  end;
end delete_table;
/
---------------------------------------------------------------------------------------------------
-- Удаляем таблицы с тестовыми данными, если они есть. Не нужно при непрерывной интеграции, но полезно при разработке,
-- когда база не откатывается каждый раз
begin
  delete_table('type_text');
  delete_table('type_number');
  delete_table('type_time');
  delete_table('type_bin');
  delete_table('type_lob');
  delete_table('type_specific');
end;
/
drop procedure delete_table;