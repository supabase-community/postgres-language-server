-- Postgres rejects a function whose body uses a missing table, so calling it doesn't create
-- anything: the tables its body would create are still missing afterwards.
-- expect_lint/unknownRelation
create function make_table() returns void language sql as $$
  create table created_table as select * from missing_table;
$$;

select make_table();

-- expect_lint/unknownRelation
table created_table;

-- The body uses the table that it creates, which doesn't exist when Postgres validates it.
-- expect_lint/unknownRelation
create function create_and_insert() returns void language sql as $$
  create table ddl_test (f1 int);
  insert into ddl_test values (1);
$$;

select create_and_insert();

-- expect_lint/unknownRelation
table ddl_test;
