-- Идиотизм: sql*plus трактует пустую строку, как завершение SQL-выражения, поэтому пустые строки в
-- определениях таблиц заполнены разделителем '--------------------------------------'.
---------------------------------------------------------------------------------------------------
create table type_text (
  id number(2) not null,-- номер теста
--------------------------------------
  -- Устаревший тип столбца
  col0 long,
--------------------------------------
  -- Минимальный размер
  col1 varchar2(1 byte),
  col2 varchar2(1 char),
--------------------------------------
  -- Максимальный размер при стандартных настройках
  col3 varchar2(4000 byte),
  col4 varchar2(4000 char),
--------------------------------------
  -- Минимальный размер
  col5 nvarchar2(1),
  -- Максимальный размер при стандартных настройках
  -- Размер задается в символах, а в ограничениях проверяются байты. Максимум равен 2000 (32767) байт,
  -- для UTF-16 это 1000 (16383) символов, для UTF-8 - ~666 (10922) символов
  col6 nvarchar2(1000),
--------------------------------------
  -- Минимальный размер
  col7 char(1 byte),
  col8 char(1 char),
  -- Максимальный размер
  col9  char(2000 byte),
  col10 char(2000 char),
--------------------------------------
  -- Минимальный размер
  col11 nchar(1),
  -- Максимальный размер при стандартных настройках
  -- Размер задается в символах, а в ограничениях проверяются байты. Максимум равен 2000 байт,
  -- для UTF-16 это 1000 символов, для UTF-8 - 2000/3 ~ 666 символов
  col12 nchar(1000)
);
insert into type_text values(0,
  -- long
  null,
  -- varchar2
  null, null,
  null, null,
  -- nvarchar2
  null, null,
  -- char
  null, null,
  null, null,
  -- nchar
  null, null
);
---------------------------------------------------------------------------------------------------
create table type_number (
  id number(2) not null,-- номер теста
--------------------------------------
  -- Минимальный размер
  col0 number(1),
  -- Максимальный размер
  col1 number(38),
--------------------------------------
  -- Минимальный размер
  col2 float(1),
  -- Максимальный размер
  col3 float(126),
--------------------------------------
  col4 binary_float,
  col5 binary_double
);
insert into type_number values(0,
  -- number
  null, null,
  -- float
  null, null,
  -- binary*
  null, null
);
---------------------------------------------------------------------------------------------------
create table type_time (
  id number(2) not null,-- номер теста
--------------------------------------
  col0 date,
--------------------------------------
  -- Минимальный размер
  col1 timestamp(0),
  col2 timestamp(0) with time zone,
  col3 timestamp(0) with local time zone,
  -- Максимальный размер
  col4 timestamp(9),
  col5 timestamp(9) with time zone,
  col6 timestamp(9) with local time zone,
--------------------------------------
  -- Минимальный размер
  col7  interval year(0) to month,
  col8  interval day(0) to second(0),
  -- Максимальный размер
  col9  interval year(9) to month,
  col10 interval day(9) to second(9)
);
insert into type_time values(0,
  -- date
  null,
  -- timestamp
  null, null, null,
  null, null, null,
  -- interval
  null, null,
  null, null
);
---------------------------------------------------------------------------------------------------
create table type_bin (
  id number(2) not null,-- номер теста
--------------------------------------
  -- Минимальный размер
  col0 raw(1),
  -- Максимальный размер при стандартных настройках
  col1 raw(2000),
  col2 long raw
);
insert into type_bin values(0,
  null, null, null
);
---------------------------------------------------------------------------------------------------
create table type_lob (
  id number(2) not null,-- номер теста
--------------------------------------
  col0 clob,
  col1 nclob,
--------------------------------------
  col2 blob,
  col3 bfile
);
insert into type_lob values(0,
  -- clob
  null, null,
  -- blob
  null, null
);
---------------------------------------------------------------------------------------------------
create table type_specific (
  id number(2) not null,-- номер теста
--------------------------------------
  col0 rowid,
--------------------------------------
  -- Минимальный размер
  col1 urowid(1),
  -- Максимальный размер
  col2 urowid(4000)
);
insert into type_specific values(0,
  -- rowid
  null,
  -- urowid
  null, null
);
