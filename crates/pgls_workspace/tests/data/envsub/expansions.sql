-- +goose Up
-- +goose ENVSUB ON
create table   ${SCHEMA}.events (id bigint, at timestamp);
insert into   ${SCHEMA}.audit_log (message) values ('${MESSAGE}');
-- +goose ENVSUB OFF
select   1;
