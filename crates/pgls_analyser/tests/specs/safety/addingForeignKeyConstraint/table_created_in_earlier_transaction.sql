-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addingForeignKeyConstraint
BEGIN;
CREATE TABLE orders (id bigint, user_id bigint);
COMMIT;
BEGIN;
ALTER TABLE orders ADD CONSTRAINT orders_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id);
COMMIT;
