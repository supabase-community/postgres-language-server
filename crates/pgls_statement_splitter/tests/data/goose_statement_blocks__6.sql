-- +goose Up
-- +goose StatementBegin
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- +goose StatementEnd

CREATE TABLE users (id bigint PRIMARY KEY, updated_at timestamptz);

-- +goose StatementBegin
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app') THEN
    CREATE ROLE app;
  END IF;
END
$$;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP FUNCTION set_updated_at();
DROP TABLE users;
-- +goose StatementEnd
-- +goose StatementBegin
CREATE FUNCTION legacy() RETURNS int AS '
  SELECT 1;
' LANGUAGE sql;
-- +goose StatementEnd
