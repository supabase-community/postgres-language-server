-- A call to a function of the file whose body changes the catalog may have created anything,
-- so nothing is missing afterwards. A call to a function whose body only writes rows doesn't
-- change the catalog: the missing table before the second call is reported.
-- expect_lint/unknownRelation
create table rows_table (f1 int);

create function insert_row() returns void language sql as $$
  insert into rows_table values (1);
$$;

select insert_row();

table missing_before_call;

set check_function_bodies to off;

create function create_table() returns void language sql as $$
  create table ddl_test (f1 int);
$$;

-- Calls a function that changes the catalog.
create function create_table_indirectly() returns void language sql as $$
  select create_table();
$$;

reset check_function_bodies;

select create_table_indirectly();

table ddl_test;
