SELECT relname, reltoastrelid <> 0 AS has_toast_table
   FROM pg_class
   WHERE oid::regclass IN ('a_star', 'c_star')
   ORDER BY 1;
