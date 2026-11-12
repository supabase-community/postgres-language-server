-- expect_no_diagnostics
-- Outside of a transaction block, Postgres rejects AND CHAIN, so it leaves no transaction open.
-- section: up transaction:false queries:separate
create table users (id bigint);
commit and chain;
