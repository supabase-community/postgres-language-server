-- serial2 and serial4 are smallint and integer columns, serial8 is a bigint.
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
-- expect_lint/preferBigInt
create table counters (
    a smallserial,
    b serial2,
    c serial,
    d serial4,
    e serial8,
    f bigserial
);
