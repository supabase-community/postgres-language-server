-- Postgres rejects arrays of a serial type ("array of serial is not implemented"), so there is
-- no serial column.
-- expect_no_diagnostics
create table counters (
    a serial[]
);
