-- smallint and integer are pg_catalog.int2 and pg_catalog.int4. myschema.int4, int4.counter
-- and the quoted "integer" are types of the user.
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
create table counters (
    a smallint,
    b int4,
    c pg_catalog.int4,
    d serial,
    e myschema.int4,
    f int4.counter,
    g "integer",
    h bigint
);
