select has_table_privilege(t1.oid,'select')
from (select oid from pg_class where relname = 'atest1') as t1;
