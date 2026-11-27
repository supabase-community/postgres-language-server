-- +goose Up
select '${A}';
-- +goose ENVSUB ON
create schema ${SCHEMA};
grant usage on schema ${SCHEMA:-${FALLBACK}} to $ROLE, \${KEPT};
-- +goose ENVSUB OFF
select ${B};
