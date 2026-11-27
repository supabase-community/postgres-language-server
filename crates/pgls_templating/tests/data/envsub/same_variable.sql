-- +goose Up
-- +goose ENVSUB ON
-- the same variable is the same value
select * from t where a = ${N} and b = ${N} and c = ${M};
