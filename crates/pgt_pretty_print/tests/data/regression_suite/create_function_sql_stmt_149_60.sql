CREATE FUNCTION alter_and_insert() RETURNS VOID LANGUAGE sql AS $$
  alter table ddl_test alter column f1 type numeric;
  insert into ddl_test values (1.2);
$$;
