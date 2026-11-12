-- Test adding regular column with integer type (should be safe)
-- expect_no_diagnostics
ALTER TABLE prices ADD COLUMN quantity integer DEFAULT 0;
-- A virtual generated column (Postgres 18) is computed on read, so there is no rewrite
ALTER TABLE prices ADD COLUMN total int GENERATED ALWAYS AS (price * quantity) VIRTUAL;
ALTER TABLE prices ADD COLUMN subtotal int GENERATED ALWAYS AS (price * quantity);
