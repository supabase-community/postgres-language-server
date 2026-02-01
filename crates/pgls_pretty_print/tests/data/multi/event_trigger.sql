create event trigger regress_event_trigger
   on ddl_command_start
   execute procedure pg_backend_pid();

create function test_event_trigger() returns event_trigger as $$
BEGIN
    RAISE NOTICE 'test_event_trigger: % %', tg_event, tg_tag;
END
$$ language plpgsql;

SELECT test_event_trigger();

create function test_event_trigger_arg(name text)
returns event_trigger as $$ BEGIN RETURN 1; END $$ language plpgsql;

create function test_event_trigger_sql() returns event_trigger as $$
SELECT 1 $$ language sql;

create event trigger regress_event_trigger on elephant_bootstrap
   execute procedure test_event_trigger();

create event trigger regress_event_trigger on ddl_command_start
   execute procedure test_event_trigger();

create event trigger regress_event_trigger_end on ddl_command_end
   execute function test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when food in ('sandwich')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('sandwich')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('create table', 'create skunkcabbage')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('DROP EVENT TRIGGER')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('CREATE ROLE')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('CREATE DATABASE')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('CREATE TABLESPACE')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('create table') and tag in ('CREATE FUNCTION')
   execute procedure test_event_trigger();

create event trigger regress_event_trigger2 on ddl_command_start
   when tag in ('create table', 'CREATE FUNCTION')
   execute procedure test_event_trigger();

comment on event trigger regress_event_trigger is 'test comment';

create role regress_evt_user;

set role regress_evt_user;

create event trigger regress_event_trigger_noperms on ddl_command_start
   execute procedure test_event_trigger();

reset role;

alter event trigger regress_event_trigger disable;

create table event_trigger_fire1 (a int);

alter event trigger regress_event_trigger enable;

set session_replication_role = replica;

create table event_trigger_fire2 (a int);

alter event trigger regress_event_trigger enable replica;

create table event_trigger_fire3 (a int);

alter event trigger regress_event_trigger enable always;

create table event_trigger_fire4 (a int);

reset session_replication_role;

create table event_trigger_fire5 (a int);

create function f1() returns int
language plpgsql
as $$
begin
  create table event_trigger_fire6 (a int);
  return 0;
end $$;

select f1();

create procedure p1()
language plpgsql
as $$
begin
  create table event_trigger_fire7 (a int);
end $$;

call p1();

alter event trigger regress_event_trigger disable;

drop table event_trigger_fire2, event_trigger_fire3, event_trigger_fire4, event_trigger_fire5, event_trigger_fire6, event_trigger_fire7;

drop routine f1(), p1();

grant all on table event_trigger_fire1 to public;

comment on table event_trigger_fire1 is 'here is a comment';

revoke all on table event_trigger_fire1 from public;

drop table event_trigger_fire1;

create foreign data wrapper useless;

create server useless_server foreign data wrapper useless;

create user mapping for regress_evt_user server useless_server;

alter default privileges for role regress_evt_user
 revoke delete on tables from regress_evt_user;

alter event trigger regress_event_trigger owner to regress_evt_user;

alter role regress_evt_user superuser;

alter event trigger regress_event_trigger owner to regress_evt_user;

alter event trigger regress_event_trigger rename to regress_event_trigger2;

alter event trigger regress_event_trigger rename to regress_event_trigger3;

drop event trigger regress_event_trigger;

drop role regress_evt_user;

drop event trigger if exists regress_event_trigger2;

drop event trigger if exists regress_event_trigger2;

drop event trigger regress_event_trigger3;

drop event trigger regress_event_trigger_end;

CREATE SCHEMA schema_one authorization regress_evt_user;

CREATE SCHEMA schema_two authorization regress_evt_user;

CREATE SCHEMA audit_tbls authorization regress_evt_user;

CREATE TEMP TABLE a_temp_tbl ();

SET SESSION AUTHORIZATION regress_evt_user;

CREATE TABLE schema_one.table_one(a int);

CREATE TABLE schema_one."table two"(a int);

CREATE TABLE schema_one.table_three(a int);

CREATE TABLE audit_tbls.schema_one_table_two(the_value text);

CREATE TABLE schema_two.table_two(a int);

CREATE TABLE schema_two.table_three(a int, b text);

CREATE TABLE audit_tbls.schema_two_table_three(the_value text);

CREATE OR REPLACE FUNCTION schema_two.add(int, int) RETURNS int LANGUAGE plpgsql
  CALLED ON NULL INPUT
  AS $$ BEGIN RETURN coalesce($1,0) + coalesce($2,0); END; $$;

CREATE AGGREGATE schema_two.newton
  (BASETYPE = int, SFUNC = schema_two.add, STYPE = int);

RESET SESSION AUTHORIZATION;

CREATE TABLE undroppable_objs (
	object_type text,
	object_identity text
);

INSERT INTO undroppable_objs VALUES
('table', 'schema_one.table_three'),
('table', 'audit_tbls.schema_two_table_three');

CREATE TABLE dropped_objects (
	object_type text,
	schema_name text,
	object_name text,
	object_identity text,
	address_names text[],
	address_args text[],
	is_temporary bool,
	original bool,
	normal bool
);

CREATE OR REPLACE FUNCTION undroppable() RETURNS event_trigger
LANGUAGE plpgsql AS $$
DECLARE
	obj record;
BEGIN
	PERFORM 1 FROM pg_tables WHERE tablename = 'undroppable_objs';
	IF NOT FOUND THEN
		RAISE NOTICE 'table undroppable_objs not found, skipping';
		RETURN;
	END IF;
	FOR obj IN
		SELECT * FROM pg_event_trigger_dropped_objects() JOIN
			undroppable_objs USING (object_type, object_identity)
	LOOP
		RAISE EXCEPTION 'object % of type % cannot be dropped',
			obj.object_identity, obj.object_type;
	END LOOP;
END;
$$;

CREATE EVENT TRIGGER undroppable ON sql_drop
	EXECUTE PROCEDURE undroppable();

CREATE OR REPLACE FUNCTION test_evtrig_dropped_objects() RETURNS event_trigger
LANGUAGE plpgsql AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_dropped_objects()
    LOOP
        IF obj.object_type = 'table' THEN
                EXECUTE format('DROP TABLE IF EXISTS audit_tbls.%I',
					format('%s_%s', obj.schema_name, obj.object_name));
        END IF;

	INSERT INTO dropped_objects
		(object_type, schema_name, object_name,
		 object_identity, address_names, address_args,
		 is_temporary, original, normal) VALUES
		(obj.object_type, obj.schema_name, obj.object_name,
		 obj.object_identity, obj.address_names, obj.address_args,
		 obj.is_temporary, obj.original, obj.normal);
    END LOOP;
END
$$;

CREATE EVENT TRIGGER regress_event_trigger_drop_objects ON sql_drop
	WHEN TAG IN ('drop table', 'drop function', 'drop view',
		'drop owned', 'drop schema', 'alter table')
	EXECUTE PROCEDURE test_evtrig_dropped_objects();

ALTER TABLE schema_one.table_one DROP COLUMN a;

DROP SCHEMA schema_one, schema_two CASCADE;

DELETE FROM undroppable_objs WHERE object_identity = 'audit_tbls.schema_two_table_three';

DROP SCHEMA schema_one, schema_two CASCADE;

DELETE FROM undroppable_objs WHERE object_identity = 'schema_one.table_three';

DROP SCHEMA schema_one, schema_two CASCADE;

SELECT * FROM dropped_objects
  WHERE schema_name IS NULL OR schema_name <> 'pg_toast';

DROP OWNED BY regress_evt_user;

SELECT * FROM dropped_objects WHERE object_type = 'schema';

DROP ROLE regress_evt_user;

DROP EVENT TRIGGER regress_event_trigger_drop_objects;

DROP EVENT TRIGGER undroppable;

CREATE OR REPLACE FUNCTION event_trigger_report_dropped()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $$
DECLARE r record;
BEGIN
    FOR r IN SELECT * from pg_event_trigger_dropped_objects()
    LOOP
    IF NOT r.normal AND NOT r.original THEN
        CONTINUE;
    END IF;
    RAISE NOTICE 'NORMAL: orig=% normal=% istemp=% type=% identity=% schema=% name=% addr=% args=%',
        r.original, r.normal, r.is_temporary, r.object_type,
        r.object_identity, r.schema_name, r.object_name,
        r.address_names, r.address_args;
    END LOOP;
END; $$;

CREATE EVENT TRIGGER regress_event_trigger_report_dropped ON sql_drop
    EXECUTE PROCEDURE event_trigger_report_dropped();

CREATE OR REPLACE FUNCTION event_trigger_report_end()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $$
DECLARE r RECORD;
BEGIN
    FOR r IN SELECT * FROM pg_event_trigger_ddl_commands()
    LOOP
        RAISE NOTICE 'END: command_tag=% type=% identity=%',
            r.command_tag, r.object_type, r.object_identity;
    END LOOP;
END; $$;

CREATE EVENT TRIGGER regress_event_trigger_report_end ON ddl_command_end
  EXECUTE PROCEDURE event_trigger_report_end();

CREATE SCHEMA evttrig

CREATE TABLE one (col_a SERIAL PRIMARY KEY, col_b text DEFAULT 'forty two', col_c SERIAL)

CREATE INDEX one_idx ON one (col_b)

CREATE TABLE two (col_c INTEGER CHECK (col_c > 0) REFERENCES one DEFAULT 42)

CREATE TABLE id (col_d int NOT NULL GENERATED ALWAYS AS IDENTITY);

CREATE TABLE evttrig.parted (
    id int PRIMARY KEY)
    PARTITION BY RANGE (id);

CREATE TABLE evttrig.part_1_10 PARTITION OF evttrig.parted (id)
  FOR VALUES FROM (1) TO (10);

CREATE TABLE evttrig.part_10_20 PARTITION OF evttrig.parted (id)
  FOR VALUES FROM (10) TO (20) PARTITION BY RANGE (id);

CREATE TABLE evttrig.part_10_15 PARTITION OF evttrig.part_10_20 (id)
  FOR VALUES FROM (10) TO (15);

CREATE TABLE evttrig.part_15_20 PARTITION OF evttrig.part_10_20 (id)
  FOR VALUES FROM (15) TO (20);

ALTER TABLE evttrig.two DROP COLUMN col_c;

ALTER TABLE evttrig.one ALTER COLUMN col_b DROP DEFAULT;

ALTER TABLE evttrig.one DROP CONSTRAINT one_pkey;

ALTER TABLE evttrig.one DROP COLUMN col_c;

ALTER TABLE evttrig.id ALTER COLUMN col_d SET DATA TYPE bigint;

ALTER TABLE evttrig.id ALTER COLUMN col_d DROP IDENTITY,
  ALTER COLUMN col_d SET DATA TYPE int;

DROP INDEX evttrig.one_idx;

DROP SCHEMA evttrig CASCADE;

DROP TABLE a_temp_tbl;

CREATE OR REPLACE FUNCTION event_trigger_report_dropped()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $$
DECLARE r record;
BEGIN
    FOR r IN SELECT * from pg_event_trigger_dropped_objects()
    LOOP
    RAISE NOTICE 'DROP: orig=% normal=% istemp=% type=% identity=% schema=% name=% addr=% args=%',
        r.original, r.normal, r.is_temporary, r.object_type,
        r.object_identity, r.schema_name, r.object_name,
        r.address_names, r.address_args;
    END LOOP;
END; $$;

CREATE FUNCTION event_trigger_dummy_trigger()
 RETURNS trigger
 LANGUAGE plpgsql
AS $$
BEGIN
    RETURN new;
END; $$;

CREATE TABLE evtrg_nontemp_table (f1 int primary key, f2 int default 42);

CREATE TRIGGER evtrg_nontemp_trig
  BEFORE INSERT ON evtrg_nontemp_table
  EXECUTE FUNCTION event_trigger_dummy_trigger();

CREATE POLICY evtrg_nontemp_pol ON evtrg_nontemp_table USING (f2 > 0);

DROP TABLE evtrg_nontemp_table;

CREATE TEMP TABLE a_temp_tbl (f1 int primary key, f2 int default 42);

CREATE TRIGGER a_temp_trig
  BEFORE INSERT ON a_temp_tbl
  EXECUTE FUNCTION event_trigger_dummy_trigger();

CREATE POLICY a_temp_pol ON a_temp_tbl USING (f2 > 0);

DROP TABLE a_temp_tbl;

DROP FUNCTION event_trigger_dummy_trigger();

CREATE OPERATOR CLASS evttrigopclass FOR TYPE int USING btree AS STORAGE int;

DROP EVENT TRIGGER regress_event_trigger_report_dropped;

DROP EVENT TRIGGER regress_event_trigger_report_end;

select pg_event_trigger_table_rewrite_oid();

CREATE OR REPLACE FUNCTION test_evtrig_no_rewrite() RETURNS event_trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'rewrites not allowed';
END;
$$;

create event trigger no_rewrite_allowed on table_rewrite
  execute procedure test_evtrig_no_rewrite();

create table rewriteme (id serial primary key, foo float, bar timestamptz);

insert into rewriteme
     select x * 1.001 from generate_series(1, 500) as t(x);

alter table rewriteme alter column foo type numeric;

alter table rewriteme add column baz int default 0;

CREATE OR REPLACE FUNCTION test_evtrig_no_rewrite() RETURNS event_trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE NOTICE 'Table ''%'' is being rewritten (reason = %)',
               pg_event_trigger_table_rewrite_oid()::regclass,
               pg_event_trigger_table_rewrite_reason();
END;
$$;

alter table rewriteme
 add column onemore int default 0,
 add column another int default -1,
 alter column foo type numeric(10,4);

CREATE MATERIALIZED VIEW heapmv USING heap AS SELECT 1 AS a;

ALTER MATERIALIZED VIEW heapmv SET ACCESS METHOD heap2;

DROP MATERIALIZED VIEW heapmv;

alter table rewriteme alter column foo type numeric(12,4);

begin;

set timezone to 'UTC';

alter table rewriteme alter column bar type timestamp;

set timezone to '0';

alter table rewriteme alter column bar type timestamptz;

set timezone to 'Europe/London';

alter table rewriteme alter column bar type timestamp;

rollback;

CREATE OR REPLACE FUNCTION test_evtrig_no_rewrite() RETURNS event_trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE NOTICE 'Table is being rewritten (reason = %)',
               pg_event_trigger_table_rewrite_reason();
END;
$$;

create type rewritetype as (a int);

create table rewritemetoo1 of rewritetype;

create table rewritemetoo2 of rewritetype;

alter type rewritetype alter attribute a type text cascade;

create table rewritemetoo3 (a rewritetype);

alter type rewritetype alter attribute a type varchar cascade;

drop table rewriteme;

drop event trigger no_rewrite_allowed;

drop function test_evtrig_no_rewrite();

CREATE OR REPLACE FUNCTION reindex_start_command()
RETURNS event_trigger AS $$
BEGIN
    RAISE NOTICE 'REINDEX START: % %', tg_event, tg_tag;
END;
$$ LANGUAGE plpgsql;

CREATE EVENT TRIGGER regress_reindex_start ON ddl_command_start
    WHEN TAG IN ('REINDEX')
    EXECUTE PROCEDURE reindex_start_command();

CREATE FUNCTION reindex_end_command()
RETURNS event_trigger AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_ddl_commands()
    LOOP
        RAISE NOTICE 'REINDEX END: command_tag=% type=% identity=%',
	    obj.command_tag, obj.object_type, obj.object_identity;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

CREATE EVENT TRIGGER regress_reindex_end ON ddl_command_end
    WHEN TAG IN ('REINDEX')
    EXECUTE PROCEDURE reindex_end_command();

CREATE FUNCTION reindex_end_command_snap() RETURNS EVENT_TRIGGER
    AS $$ BEGIN PERFORM 1; END $$ LANGUAGE plpgsql;

CREATE EVENT TRIGGER regress_reindex_end_snap ON ddl_command_end
    EXECUTE FUNCTION reindex_end_command_snap();

CREATE TABLE concur_reindex_tab (c1 int);

CREATE INDEX concur_reindex_ind ON concur_reindex_tab (c1);

REINDEX INDEX concur_reindex_ind;

REINDEX TABLE concur_reindex_tab;

REINDEX INDEX CONCURRENTLY concur_reindex_ind;

REINDEX TABLE CONCURRENTLY concur_reindex_tab;

ALTER EVENT TRIGGER regress_reindex_start DISABLE;

REINDEX INDEX concur_reindex_ind;

REINDEX INDEX CONCURRENTLY concur_reindex_ind;

DROP INDEX concur_reindex_ind;

REINDEX TABLE concur_reindex_tab;

REINDEX TABLE CONCURRENTLY concur_reindex_tab;

CREATE SCHEMA concur_reindex_schema;

REINDEX SCHEMA concur_reindex_schema;

REINDEX SCHEMA CONCURRENTLY concur_reindex_schema;

CREATE TABLE concur_reindex_schema.tab (a int);

CREATE INDEX ind ON concur_reindex_schema.tab (a);

REINDEX SCHEMA concur_reindex_schema;

REINDEX SCHEMA CONCURRENTLY concur_reindex_schema;

DROP INDEX concur_reindex_schema.ind;

REINDEX SCHEMA concur_reindex_schema;

REINDEX SCHEMA CONCURRENTLY concur_reindex_schema;

DROP SCHEMA concur_reindex_schema CASCADE;

CREATE TABLE concur_reindex_part (id int) PARTITION BY RANGE (id);

REINDEX TABLE concur_reindex_part;

REINDEX TABLE CONCURRENTLY concur_reindex_part;

CREATE TABLE concur_reindex_child PARTITION OF concur_reindex_part
  FOR VALUES FROM (0) TO (10);

REINDEX TABLE concur_reindex_part;

REINDEX TABLE CONCURRENTLY concur_reindex_part;

CREATE INDEX concur_reindex_partidx ON concur_reindex_part (id);

REINDEX INDEX concur_reindex_partidx;

REINDEX INDEX CONCURRENTLY concur_reindex_partidx;

REINDEX TABLE concur_reindex_part;

REINDEX TABLE CONCURRENTLY concur_reindex_part;

DROP TABLE concur_reindex_part;

DROP EVENT TRIGGER regress_reindex_start;

DROP EVENT TRIGGER regress_reindex_end;

DROP EVENT TRIGGER regress_reindex_end_snap;

DROP FUNCTION reindex_end_command();

DROP FUNCTION reindex_end_command_snap();

DROP FUNCTION reindex_start_command();

DROP TABLE concur_reindex_tab;

RESET SESSION AUTHORIZATION;

CREATE TABLE event_trigger_test (a integer, b text);

CREATE OR REPLACE FUNCTION start_command()
RETURNS event_trigger AS $$
BEGIN
RAISE NOTICE '% - ddl_command_start', tg_tag;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION end_command()
RETURNS event_trigger AS $$
BEGIN
RAISE NOTICE '% - ddl_command_end', tg_tag;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION drop_sql_command()
RETURNS event_trigger AS $$
BEGIN
RAISE NOTICE '% - sql_drop', tg_tag;
END;
$$ LANGUAGE plpgsql;

CREATE EVENT TRIGGER start_rls_command ON ddl_command_start
    WHEN TAG IN ('CREATE POLICY', 'ALTER POLICY', 'DROP POLICY') EXECUTE PROCEDURE start_command();

CREATE EVENT TRIGGER end_rls_command ON ddl_command_end
    WHEN TAG IN ('CREATE POLICY', 'ALTER POLICY', 'DROP POLICY') EXECUTE PROCEDURE end_command();

CREATE EVENT TRIGGER sql_drop_command ON sql_drop
    WHEN TAG IN ('DROP POLICY') EXECUTE PROCEDURE drop_sql_command();

CREATE POLICY p1 ON event_trigger_test USING (FALSE);

ALTER POLICY p1 ON event_trigger_test USING (TRUE);

ALTER POLICY p1 ON event_trigger_test RENAME TO p2;

DROP POLICY p2 ON event_trigger_test;

SELECT
    e.evtname,
    pg_describe_object('pg_event_trigger'::regclass, e.oid, 0) as descr,
    b.type, b.object_names, b.object_args,
    pg_identify_object(a.classid, a.objid, a.objsubid) as ident
  FROM pg_event_trigger as e,
    LATERAL pg_identify_object_as_address('pg_event_trigger'::regclass, e.oid, 0) as b,
    LATERAL pg_get_object_address(b.type, b.object_names, b.object_args) as a
  ORDER BY e.evtname;

DROP EVENT TRIGGER start_rls_command;

DROP EVENT TRIGGER end_rls_command;

DROP EVENT TRIGGER sql_drop_command;

CREATE FUNCTION test_event_trigger_guc() RETURNS event_trigger
LANGUAGE plpgsql AS $$
DECLARE
	obj record;
BEGIN
	FOR obj IN SELECT * FROM pg_event_trigger_dropped_objects()
	LOOP
		RAISE NOTICE '% dropped %', tg_tag, obj.object_type;
	END LOOP;
END;
$$;

CREATE EVENT TRIGGER test_event_trigger_guc
	ON sql_drop
	WHEN TAG IN ('DROP POLICY') EXECUTE FUNCTION test_event_trigger_guc();

SET event_triggers = 'on';

CREATE POLICY pguc ON event_trigger_test USING (FALSE);

DROP POLICY pguc ON event_trigger_test;

CREATE POLICY pguc ON event_trigger_test USING (FALSE);

SET event_triggers = 'off';

DROP POLICY pguc ON event_trigger_test;
