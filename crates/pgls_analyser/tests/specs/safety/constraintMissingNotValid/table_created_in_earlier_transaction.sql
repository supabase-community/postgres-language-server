-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/constraintMissingNotValid
BEGIN;
CREATE TABLE orders (id bigint, amount int);
COMMIT;
BEGIN;
ALTER TABLE orders ADD CONSTRAINT orders_amount_check CHECK (amount > 0);
COMMIT;
