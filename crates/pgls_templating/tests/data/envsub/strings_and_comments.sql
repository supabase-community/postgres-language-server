-- +goose Up
-- +goose ENVSUB ON
insert into audit_log (message) values ('${MESSAGE}'); -- ${NOTE}
