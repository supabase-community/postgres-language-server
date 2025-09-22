CREATE FUNCTION create_and_insert() RETURNS VOID LANGUAGE sql AS $$
  create table ddl_test (f1 int);
  insert into ddl_test values (1.2);
$$;
