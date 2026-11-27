-- +goose Up
-- +goose ENVSUB ON
create table ${PREFIX}_events (id bigint);
select * from "${SCHEMA}".events, app_$PREFIX.${A}${B};
