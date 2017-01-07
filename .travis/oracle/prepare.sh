#!/bin/sh -e

"$ORACLE_HOME/bin/sqlplus" -L -S / @.travis/oracle/cleanup.sql
"$ORACLE_HOME/bin/sqlplus" -L -S / <<SQL
create or replace directory test_bfiles as '`pwd`/tests/resources';
SQL
"$ORACLE_HOME/bin/sqlplus" -L -S / @.travis/oracle/prepare.sql
"$ORACLE_HOME/bin/sqlplus" -L -S / as sysdba @.travis/oracle/prepare-doc-examples.sql