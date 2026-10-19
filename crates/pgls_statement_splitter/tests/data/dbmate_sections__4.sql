-- migrate:up transaction:false
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
ALTER TABLE users ADD COLUMN name text;

-- migrate:down
DROP INDEX users_email_idx;
ALTER TABLE users DROP COLUMN name;
