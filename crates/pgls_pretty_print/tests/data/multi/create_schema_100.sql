CREATE ROLE regress_create_schema_role SUPERUSER;

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE SEQUENCE schema_not_existing.seq;

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE TABLE schema_not_existing.tab (id int);

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE VIEW schema_not_existing.view AS SELECT 1;

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE INDEX ON schema_not_existing.tab (id);

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE TRIGGER schema_trig BEFORE INSERT ON schema_not_existing.tab
  EXECUTE FUNCTION schema_trig.no_func();

SET ROLE regress_create_schema_role;

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE SEQUENCE schema_not_existing.seq;

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE TABLE schema_not_existing.tab (id int);

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE VIEW schema_not_existing.view AS SELECT 1;

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE INDEX ON schema_not_existing.tab (id);

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE TRIGGER schema_trig BEFORE INSERT ON schema_not_existing.tab
  EXECUTE FUNCTION schema_trig.no_func();

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE SEQUENCE schema_not_existing.seq;

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE TABLE schema_not_existing.tab (id int);

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE VIEW schema_not_existing.view AS SELECT 1;

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE INDEX ON schema_not_existing.tab (id);

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE TRIGGER schema_trig BEFORE INSERT ON schema_not_existing.tab
  EXECUTE FUNCTION schema_trig.no_func();

RESET ROLE;

CREATE SCHEMA AUTHORIZATION regress_create_schema_role

CREATE TABLE regress_create_schema_role.tab (id int);

DROP SCHEMA regress_create_schema_role CASCADE;

SET ROLE regress_create_schema_role;

CREATE SCHEMA AUTHORIZATION CURRENT_ROLE

CREATE TABLE regress_create_schema_role.tab (id int);

DROP SCHEMA regress_create_schema_role CASCADE;

CREATE SCHEMA regress_schema_1 AUTHORIZATION CURRENT_ROLE

CREATE TABLE regress_schema_1.tab (id int);

DROP SCHEMA regress_schema_1 CASCADE;

RESET ROLE;

DROP ROLE regress_create_schema_role;
