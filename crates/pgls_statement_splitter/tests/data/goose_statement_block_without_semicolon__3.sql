-- +goose Up
-- +goose StatementBegin
CREATE FUNCTION one() RETURNS int AS $$
BEGIN
  RETURN 1;
END;
$$ LANGUAGE plpgsql
-- +goose StatementEnd
CREATE TABLE t (id int);

-- +goose Down
DROP TABLE t;
