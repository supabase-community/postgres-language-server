-- +goose Up
-- +goose ENVSUB ON
create schema ${SCHEMA:-app{test};
-- interpolate rejects this line, so goose fails on it, and it stays as written
select ${A:?};
