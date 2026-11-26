-- A storage parameter name that needs quotes keeps them.
CREATE TABLE t WITH ("Fill*factor" = 10, toast.autovacuum_enabled = false) AS SELECT 1 a;
