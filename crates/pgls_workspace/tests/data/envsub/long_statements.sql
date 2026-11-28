-- +goose Up
-- +goose ENVSUB ON
select id, name, email, created_at, updated_at, deleted_at from ${SCHEMA}.users where tenant_id = ${TENANT} and created_at > ${SINCE} order by created_at desc;
