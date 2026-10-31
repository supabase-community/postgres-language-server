-- Unqualified tables are in the first schema of the search path: app.users is new, app.orders
-- is not.
-- expect_lint/lockTimeoutWarning
SET search_path = app, public;
CREATE TABLE users (id bigint);
ALTER TABLE users ADD COLUMN email text;
ALTER TABLE orders ADD COLUMN note text;
