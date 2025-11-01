CREATE USER regress_dep_user;

CREATE USER regress_dep_user2;

CREATE USER regress_dep_user3;

CREATE GROUP regress_dep_group;

CREATE TABLE deptest (f1 serial primary key, f2 text);

GRANT SELECT ON TABLE deptest TO GROUP regress_dep_group;

GRANT ALL ON TABLE deptest TO regress_dep_user, regress_dep_user2;

DROP USER regress_dep_user;

DROP GROUP regress_dep_group;

REVOKE SELECT ON deptest FROM GROUP regress_dep_group;

DROP GROUP regress_dep_group;

REVOKE SELECT, INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, MAINTAIN ON deptest FROM regress_dep_user;

DROP USER regress_dep_user;

REVOKE TRIGGER ON deptest FROM regress_dep_user;

DROP USER regress_dep_user;

REVOKE ALL ON deptest FROM regress_dep_user2;

DROP USER regress_dep_user2;

ALTER TABLE deptest OWNER TO regress_dep_user3;

DROP USER regress_dep_user3;

DROP TABLE deptest;

DROP USER regress_dep_user3;

CREATE USER regress_dep_user0;

CREATE USER regress_dep_user1;

CREATE USER regress_dep_user2;

SET SESSION AUTHORIZATION regress_dep_user0;

DROP OWNED BY regress_dep_user1;

DROP OWNED BY regress_dep_user0, regress_dep_user2;

REASSIGN OWNED BY regress_dep_user0 TO regress_dep_user1;

REASSIGN OWNED BY regress_dep_user1 TO regress_dep_user0;

DROP OWNED BY regress_dep_user0;

CREATE TABLE deptest1 (f1 int unique);

GRANT ALL ON deptest1 TO regress_dep_user1 WITH GRANT OPTION;

SET SESSION AUTHORIZATION regress_dep_user1;

CREATE TABLE deptest (a serial primary key, b text);

GRANT ALL ON deptest1 TO regress_dep_user2;

RESET SESSION AUTHORIZATION;

DROP OWNED BY regress_dep_user1;

GRANT ALL ON deptest1 TO regress_dep_user1;

GRANT CREATE ON DATABASE regression TO regress_dep_user1;

SET SESSION AUTHORIZATION regress_dep_user1;

CREATE SCHEMA deptest;

CREATE TABLE deptest (a serial primary key, b text);

ALTER DEFAULT PRIVILEGES FOR ROLE regress_dep_user1 IN SCHEMA deptest
  GRANT ALL ON TABLES TO regress_dep_user2;

CREATE FUNCTION deptest_func() RETURNS void LANGUAGE plpgsql
  AS $$ BEGIN END; $$;

CREATE TYPE deptest_enum AS ENUM ('red');

CREATE TYPE deptest_range AS RANGE (SUBTYPE = int4);

CREATE TABLE deptest2 (f1 int);

CREATE SEQUENCE ss1;

ALTER TABLE deptest2 ALTER f1 SET DEFAULT nextval('ss1');

ALTER SEQUENCE ss1 OWNED BY deptest2.f1;

CREATE TYPE deptest_t AS (a int);

SELECT typowner = relowner
FROM pg_type JOIN pg_class c ON typrelid = c.oid WHERE typname = 'deptest_t';

RESET SESSION AUTHORIZATION;

REASSIGN OWNED BY regress_dep_user1 TO regress_dep_user2;

SELECT typowner = relowner
FROM pg_type JOIN pg_class c ON typrelid = c.oid WHERE typname = 'deptest_t';

DROP USER regress_dep_user1;

DROP OWNED BY regress_dep_user1;

DROP USER regress_dep_user1;

DROP USER regress_dep_user2;

DROP OWNED BY regress_dep_user2, regress_dep_user0;

DROP USER regress_dep_user2;

DROP USER regress_dep_user0;
