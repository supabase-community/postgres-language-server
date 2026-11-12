-- Like Postgres, only an unqualified name is a serial type. pg_catalog.serial and
-- pg_catalog.serial2 don't exist.
-- expect_no_diagnostics
create table counters (
    a pg_catalog.serial,
    b pg_catalog.serial2,
    c pg_catalog.serial4
);
