CREATE FUNCTION trigger_return_old ()
        RETURNS trigger
        AS 'regresslib'
        LANGUAGE C;

create table trigtest (f1 int, f2 text);

create trigger trigger_return_old
	before insert or delete or update on trigtest
	for each row execute procedure trigger_return_old();

insert into trigtest values(1, 'foo');

select * from trigtest;

update trigtest set f2 = f2 || 'bar';

select * from trigtest;

delete from trigtest;

select * from trigtest;

create function f1_times_10() returns trigger as
$$ begin new.f1 := new.f1 * 10; return new; end $$ language plpgsql;

create trigger trigger_alpha
	before insert or update on trigtest
	for each row execute procedure f1_times_10();

insert into trigtest values(1, 'foo');

select * from trigtest;

update trigtest set f2 = f2 || 'bar';

select * from trigtest;

delete from trigtest;

select * from trigtest;

create trigger trigger_zed
	before insert or update on trigtest
	for each row execute procedure f1_times_10();

insert into trigtest values(1, 'foo');

select * from trigtest;

update trigtest set f2 = f2 || 'bar';

select * from trigtest;

delete from trigtest;

select * from trigtest;

drop trigger trigger_alpha on trigtest;

insert into trigtest values(1, 'foo');

select * from trigtest;

update trigtest set f2 = f2 || 'bar';

select * from trigtest;

delete from trigtest;

select * from trigtest;

drop table trigtest;

create table trigtest (
  a integer,
  b bool default true not null,
  c text default 'xyzzy' not null);

create trigger trigger_return_old
	before insert or delete or update on trigtest
	for each row execute procedure trigger_return_old();

insert into trigtest values(1);

select * from trigtest;

alter table trigtest add column d integer default 42 not null;

select * from trigtest;

update trigtest set a = 2 where a = 1 returning *;

select * from trigtest;

alter table trigtest drop column b;

select * from trigtest;

update trigtest set a = 2 where a = 1 returning *;

select * from trigtest;

drop table trigtest;

CREATE TABLE log_table (tstamp timestamp default timeofday()::timestamp);

CREATE TABLE main_table (a int unique, b int);

CREATE FUNCTION trigger_func() RETURNS trigger LANGUAGE plpgsql AS '
BEGIN
	RAISE NOTICE ''trigger_func(%) called: action = %, when = %, level = %'', TG_ARGV[0], TG_OP, TG_WHEN, TG_LEVEL;
	RETURN NULL;
END;';

CREATE TRIGGER before_ins_stmt_trig BEFORE INSERT ON main_table
FOR EACH STATEMENT EXECUTE PROCEDURE trigger_func('before_ins_stmt');

CREATE TRIGGER after_ins_stmt_trig AFTER INSERT ON main_table
FOR EACH STATEMENT EXECUTE PROCEDURE trigger_func('after_ins_stmt');

CREATE TRIGGER after_upd_stmt_trig AFTER UPDATE ON main_table
EXECUTE PROCEDURE trigger_func('after_upd_stmt');

INSERT INTO main_table (a, b) VALUES (5, 10) ON CONFLICT (a)
  DO UPDATE SET b = EXCLUDED.b;

CREATE TRIGGER after_upd_row_trig AFTER UPDATE ON main_table
FOR EACH ROW EXECUTE PROCEDURE trigger_func('after_upd_row');

INSERT INTO main_table DEFAULT VALUES;

UPDATE main_table SET a = a + 1 WHERE b < 30;

UPDATE main_table SET a = a + 2 WHERE b > 100;

ALTER TABLE main_table DROP CONSTRAINT main_table_a_key;

SELECT * FROM main_table ORDER BY a, b;

COMMENT ON TRIGGER no_such_trigger ON main_table IS 'wrong';

COMMENT ON TRIGGER before_ins_stmt_trig ON main_table IS 'right';

COMMENT ON TRIGGER before_ins_stmt_trig ON main_table IS NULL;

CREATE TRIGGER modified_a BEFORE UPDATE OF a ON main_table
FOR EACH ROW WHEN (OLD.a <> NEW.a) EXECUTE PROCEDURE trigger_func('modified_a');

CREATE TRIGGER modified_any BEFORE UPDATE OF a ON main_table
FOR EACH ROW WHEN (OLD.* IS DISTINCT FROM NEW.*) EXECUTE PROCEDURE trigger_func('modified_any');

CREATE TRIGGER insert_a AFTER INSERT ON main_table
FOR EACH ROW WHEN (NEW.a = 123) EXECUTE PROCEDURE trigger_func('insert_a');

CREATE TRIGGER delete_a AFTER DELETE ON main_table
FOR EACH ROW WHEN (OLD.a = 123) EXECUTE PROCEDURE trigger_func('delete_a');

CREATE TRIGGER insert_when BEFORE INSERT ON main_table
FOR EACH STATEMENT WHEN (true) EXECUTE PROCEDURE trigger_func('insert_when');

CREATE TRIGGER delete_when AFTER DELETE ON main_table
FOR EACH STATEMENT WHEN (true) EXECUTE PROCEDURE trigger_func('delete_when');

SELECT trigger_name, event_manipulation, event_object_schema, event_object_table,
       action_order, action_condition, action_orientation, action_timing,
       action_reference_old_table, action_reference_new_table
  FROM information_schema.triggers
  WHERE event_object_table IN ('main_table')
  ORDER BY trigger_name COLLATE "C", 2;

INSERT INTO main_table (a) VALUES (123), (456);

DELETE FROM main_table WHERE a IN (123, 456);

UPDATE main_table SET a = 50, b = 60;

SELECT * FROM main_table ORDER BY a, b;

SELECT pg_get_triggerdef(oid, true) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'modified_a';

SELECT pg_get_triggerdef(oid, false) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'modified_a';

SELECT pg_get_triggerdef(oid, true) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'modified_any';

ALTER TRIGGER modified_a ON main_table RENAME TO modified_modified_a;

SELECT count(*) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'modified_a';

SELECT count(*) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'modified_modified_a';

DROP TRIGGER modified_modified_a ON main_table;

DROP TRIGGER modified_any ON main_table;

DROP TRIGGER insert_a ON main_table;

DROP TRIGGER delete_a ON main_table;

DROP TRIGGER insert_when ON main_table;

DROP TRIGGER delete_when ON main_table;

create table table_with_oids(a int);

insert into table_with_oids values (1);

create trigger oid_unchanged_trig after update on table_with_oids
	for each row
	when (new.tableoid = old.tableoid AND new.tableoid <> 0)
	execute procedure trigger_func('after_upd_oid_unchanged');

update table_with_oids set a = a + 1;

drop table table_with_oids;

DROP TRIGGER after_upd_row_trig ON main_table;

CREATE TRIGGER before_upd_a_row_trig BEFORE UPDATE OF a ON main_table
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_upd_a_row');

CREATE TRIGGER after_upd_b_row_trig AFTER UPDATE OF b ON main_table
FOR EACH ROW EXECUTE PROCEDURE trigger_func('after_upd_b_row');

CREATE TRIGGER after_upd_a_b_row_trig AFTER UPDATE OF a, b ON main_table
FOR EACH ROW EXECUTE PROCEDURE trigger_func('after_upd_a_b_row');

CREATE TRIGGER before_upd_a_stmt_trig BEFORE UPDATE OF a ON main_table
FOR EACH STATEMENT EXECUTE PROCEDURE trigger_func('before_upd_a_stmt');

CREATE TRIGGER after_upd_b_stmt_trig AFTER UPDATE OF b ON main_table
FOR EACH STATEMENT EXECUTE PROCEDURE trigger_func('after_upd_b_stmt');

SELECT pg_get_triggerdef(oid) FROM pg_trigger WHERE tgrelid = 'main_table'::regclass AND tgname = 'after_upd_a_b_row_trig';

UPDATE main_table SET a = 50;

UPDATE main_table SET b = 10;

CREATE TABLE some_t (some_col boolean NOT NULL);

CREATE FUNCTION dummy_update_func() RETURNS trigger AS $$
BEGIN
  RAISE NOTICE 'dummy_update_func(%) called: action = %, old = %, new = %',
    TG_ARGV[0], TG_OP, OLD, NEW;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER some_trig_before BEFORE UPDATE ON some_t FOR EACH ROW
  EXECUTE PROCEDURE dummy_update_func('before');

CREATE TRIGGER some_trig_aftera AFTER UPDATE ON some_t FOR EACH ROW
  WHEN (NOT OLD.some_col AND NEW.some_col)
  EXECUTE PROCEDURE dummy_update_func('aftera');

CREATE TRIGGER some_trig_afterb AFTER UPDATE ON some_t FOR EACH ROW
  WHEN (NOT NEW.some_col)
  EXECUTE PROCEDURE dummy_update_func('afterb');

INSERT INTO some_t VALUES (TRUE);

UPDATE some_t SET some_col = TRUE;

UPDATE some_t SET some_col = FALSE;

UPDATE some_t SET some_col = TRUE;

DROP TABLE some_t;

CREATE TRIGGER error_upd_a_a BEFORE UPDATE OF a, a ON main_table
FOR EACH ROW EXECUTE PROCEDURE trigger_func('error_upd_a_a');

CREATE TRIGGER error_ins_when BEFORE INSERT OR UPDATE ON main_table
FOR EACH ROW WHEN (OLD.a <> NEW.a)
EXECUTE PROCEDURE trigger_func('error_ins_old');

CREATE TRIGGER error_del_when BEFORE DELETE OR UPDATE ON main_table
FOR EACH ROW WHEN (OLD.a <> NEW.a)
EXECUTE PROCEDURE trigger_func('error_del_new');

CREATE TRIGGER error_del_when BEFORE INSERT OR UPDATE ON main_table
FOR EACH ROW WHEN (NEW.tableoid <> 0)
EXECUTE PROCEDURE trigger_func('error_when_sys_column');

CREATE TRIGGER error_stmt_when BEFORE UPDATE OF a ON main_table
FOR EACH STATEMENT WHEN (OLD.* IS DISTINCT FROM NEW.*)
EXECUTE PROCEDURE trigger_func('error_stmt_when');

ALTER TABLE main_table DROP COLUMN b;

begin;

DROP TRIGGER after_upd_a_b_row_trig ON main_table;

DROP TRIGGER after_upd_b_row_trig ON main_table;

DROP TRIGGER after_upd_b_stmt_trig ON main_table;

ALTER TABLE main_table DROP COLUMN b;

rollback;

create table trigtest (i serial primary key);

create table trigtest2 (i int references trigtest(i) on delete cascade);

create function trigtest() returns trigger as $$
begin
	raise notice '% % % %', TG_TABLE_NAME, TG_OP, TG_WHEN, TG_LEVEL;
	return new;
end;$$ language plpgsql;

create trigger trigtest_b_row_tg before insert or update or delete on trigtest
for each row execute procedure trigtest();

create trigger trigtest_a_row_tg after insert or update or delete on trigtest
for each row execute procedure trigtest();

create trigger trigtest_b_stmt_tg before insert or update or delete on trigtest
for each statement execute procedure trigtest();

create trigger trigtest_a_stmt_tg after insert or update or delete on trigtest
for each statement execute procedure trigtest();

insert into trigtest default values;

alter table trigtest disable trigger trigtest_b_row_tg;

insert into trigtest default values;

alter table trigtest disable trigger user;

insert into trigtest default values;

alter table trigtest enable trigger trigtest_a_stmt_tg;

insert into trigtest default values;

set session_replication_role = replica;

insert into trigtest default values;

alter table trigtest enable always trigger trigtest_a_stmt_tg;

insert into trigtest default values;

reset session_replication_role;

insert into trigtest2 values(1);

insert into trigtest2 values(2);

delete from trigtest where i=2;

select * from trigtest2;

alter table trigtest disable trigger all;

delete from trigtest where i=1;

select * from trigtest2;

insert into trigtest default values;

select *  from trigtest;

drop table trigtest2;

drop table trigtest;

CREATE TABLE trigger_test (
        i int,
        v varchar
);

CREATE OR REPLACE FUNCTION trigger_data()  RETURNS trigger
LANGUAGE plpgsql AS $$

declare

	argstr text;
	relid text;

begin

	relid := TG_relid::regclass;

	-- plpgsql can't discover its trigger data in a hash like perl and python
	-- can, or by a sort of reflection like tcl can,
	-- so we have to hard code the names.
	raise NOTICE 'TG_NAME: %', TG_name;
	raise NOTICE 'TG_WHEN: %', TG_when;
	raise NOTICE 'TG_LEVEL: %', TG_level;
	raise NOTICE 'TG_OP: %', TG_op;
	raise NOTICE 'TG_RELID::regclass: %', relid;
	raise NOTICE 'TG_RELNAME: %', TG_relname;
	raise NOTICE 'TG_TABLE_NAME: %', TG_table_name;
	raise NOTICE 'TG_TABLE_SCHEMA: %', TG_table_schema;
	raise NOTICE 'TG_NARGS: %', TG_nargs;

	argstr := '[';
	for i in 0 .. TG_nargs - 1 loop
		if i > 0 then
			argstr := argstr || ', ';
		end if;
		argstr := argstr || TG_argv[i];
	end loop;
	argstr := argstr || ']';
	raise NOTICE 'TG_ARGV: %', argstr;

	if TG_OP != 'INSERT' then
		raise NOTICE 'OLD: %', OLD;
	end if;

	if TG_OP != 'DELETE' then
		raise NOTICE 'NEW: %', NEW;
	end if;

	if TG_OP = 'DELETE' then
		return OLD;
	else
		return NEW;
	end if;

end;
$$;

CREATE TRIGGER show_trigger_data_trig
BEFORE INSERT OR UPDATE OR DELETE ON trigger_test
FOR EACH ROW EXECUTE PROCEDURE trigger_data(23,'skidoo');

insert into trigger_test values(1,'insert');

update trigger_test set v = 'update' where i = 1;

delete from trigger_test;

DROP TRIGGER show_trigger_data_trig on trigger_test;

DROP FUNCTION trigger_data();

DROP TABLE trigger_test;

CREATE TABLE trigger_test (f1 int, f2 text, f3 text);

CREATE FUNCTION mytrigger() RETURNS trigger LANGUAGE plpgsql as $$
begin
	if row(old.*) = row(new.*) then
		raise notice 'row % not changed', new.f1;
	else
		raise notice 'row % changed', new.f1;
	end if;
	return new;
end$$;

CREATE TRIGGER t
BEFORE UPDATE ON trigger_test
FOR EACH ROW EXECUTE PROCEDURE mytrigger();

INSERT INTO trigger_test VALUES(1, 'foo', 'bar');

INSERT INTO trigger_test VALUES(2, 'baz', 'quux');

UPDATE trigger_test SET f3 = 'bar';

UPDATE trigger_test SET f3 = NULL;

UPDATE trigger_test SET f3 = NULL;

CREATE OR REPLACE FUNCTION mytrigger() RETURNS trigger LANGUAGE plpgsql as $$
begin
	if row(old.*) is distinct from row(new.*) then
		raise notice 'row % changed', new.f1;
	else
		raise notice 'row % not changed', new.f1;
	end if;
	return new;
end$$;

UPDATE trigger_test SET f3 = 'bar';

UPDATE trigger_test SET f3 = NULL;

UPDATE trigger_test SET f3 = NULL;

DROP TABLE trigger_test;

DROP FUNCTION mytrigger();

CREATE FUNCTION serializable_update_trig() RETURNS trigger LANGUAGE plpgsql AS
$$
declare
	rec record;
begin
	new.description = 'updated in trigger';
	return new;
end;
$$;

CREATE TABLE serializable_update_tab (
	id int,
	filler  text,
	description text
);

CREATE TRIGGER serializable_update_trig BEFORE UPDATE ON serializable_update_tab
	FOR EACH ROW EXECUTE PROCEDURE serializable_update_trig();

INSERT INTO serializable_update_tab SELECT a, repeat('xyzxz', 100), 'new'
	FROM generate_series(1, 50) a;

BEGIN;

SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;

UPDATE serializable_update_tab SET description = 'no no', id = 1 WHERE id = 1;

COMMIT;

SELECT description FROM serializable_update_tab WHERE id = 1;

DROP TABLE serializable_update_tab;

CREATE TABLE min_updates_test (
	f1	text,
	f2 int,
	f3 int);

INSERT INTO min_updates_test VALUES ('a',1,2),('b','2',null);

CREATE TRIGGER z_min_update
BEFORE UPDATE ON min_updates_test
FOR EACH ROW EXECUTE PROCEDURE suppress_redundant_updates_trigger();

UPDATE min_updates_test SET f1 = f1;

UPDATE min_updates_test SET f2 = f2 + 1;

UPDATE min_updates_test SET f3 = 2 WHERE f3 is null;

SELECT * FROM min_updates_test;

DROP TABLE min_updates_test;

CREATE VIEW main_view AS SELECT a, b FROM main_table;

CREATE OR REPLACE FUNCTION view_trigger() RETURNS trigger
LANGUAGE plpgsql AS $$
declare
    argstr text := '';
begin
    for i in 0 .. TG_nargs - 1 loop
        if i > 0 then
            argstr := argstr || ', ';
        end if;
        argstr := argstr || TG_argv[i];
    end loop;

    raise notice '% % % % (%)', TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL, argstr;

    if TG_LEVEL = 'ROW' then
        if TG_OP = 'INSERT' then
            raise NOTICE 'NEW: %', NEW;
            INSERT INTO main_table VALUES (NEW.a, NEW.b);
            RETURN NEW;
        end if;

        if TG_OP = 'UPDATE' then
            raise NOTICE 'OLD: %, NEW: %', OLD, NEW;
            UPDATE main_table SET a = NEW.a, b = NEW.b WHERE a = OLD.a AND b = OLD.b;
            if NOT FOUND then RETURN NULL; end if;
            RETURN NEW;
        end if;

        if TG_OP = 'DELETE' then
            raise NOTICE 'OLD: %', OLD;
            DELETE FROM main_table WHERE a = OLD.a AND b = OLD.b;
            if NOT FOUND then RETURN NULL; end if;
            RETURN OLD;
        end if;
    end if;

    RETURN NULL;
end;
$$;

CREATE TRIGGER invalid_trig BEFORE INSERT ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_ins_row');

CREATE TRIGGER invalid_trig BEFORE UPDATE ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_upd_row');

CREATE TRIGGER invalid_trig BEFORE DELETE ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_del_row');

CREATE TRIGGER invalid_trig AFTER INSERT ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_ins_row');

CREATE TRIGGER invalid_trig AFTER UPDATE ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_upd_row');

CREATE TRIGGER invalid_trig AFTER DELETE ON main_view
FOR EACH ROW EXECUTE PROCEDURE trigger_func('before_del_row');

CREATE TRIGGER invalid_trig BEFORE TRUNCATE ON main_view
EXECUTE PROCEDURE trigger_func('before_tru_row');

CREATE TRIGGER invalid_trig AFTER TRUNCATE ON main_view
EXECUTE PROCEDURE trigger_func('before_tru_row');

CREATE TRIGGER invalid_trig INSTEAD OF INSERT ON main_table
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_ins');

CREATE TRIGGER invalid_trig INSTEAD OF UPDATE ON main_table
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_upd');

CREATE TRIGGER invalid_trig INSTEAD OF DELETE ON main_table
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_del');

CREATE TRIGGER invalid_trig INSTEAD OF UPDATE ON main_view
FOR EACH ROW WHEN (OLD.a <> NEW.a) EXECUTE PROCEDURE view_trigger('instead_of_upd');

CREATE TRIGGER invalid_trig INSTEAD OF UPDATE OF a ON main_view
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_upd');

CREATE TRIGGER invalid_trig INSTEAD OF UPDATE ON main_view
EXECUTE PROCEDURE view_trigger('instead_of_upd');

CREATE TRIGGER instead_of_insert_trig INSTEAD OF INSERT ON main_view
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_ins');

CREATE TRIGGER instead_of_update_trig INSTEAD OF UPDATE ON main_view
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_upd');

CREATE TRIGGER instead_of_delete_trig INSTEAD OF DELETE ON main_view
FOR EACH ROW EXECUTE PROCEDURE view_trigger('instead_of_del');

CREATE TRIGGER before_ins_stmt_trig BEFORE INSERT ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('before_view_ins_stmt');

CREATE TRIGGER before_upd_stmt_trig BEFORE UPDATE ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('before_view_upd_stmt');

CREATE TRIGGER before_del_stmt_trig BEFORE DELETE ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('before_view_del_stmt');

CREATE TRIGGER after_ins_stmt_trig AFTER INSERT ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('after_view_ins_stmt');

CREATE TRIGGER after_upd_stmt_trig AFTER UPDATE ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('after_view_upd_stmt');

CREATE TRIGGER after_del_stmt_trig AFTER DELETE ON main_view
FOR EACH STATEMENT EXECUTE PROCEDURE view_trigger('after_view_del_stmt');

INSERT INTO main_view VALUES (20, 30);

INSERT INTO main_view VALUES (21, 31) RETURNING a, b;

UPDATE main_view SET b = 31 WHERE a = 20;

UPDATE main_view SET b = 32 WHERE a = 21 AND b = 31 RETURNING a, b;

DROP TRIGGER before_upd_a_row_trig ON main_table;

UPDATE main_view SET b = 31 WHERE a = 20;

UPDATE main_view SET b = 32 WHERE a = 21 AND b = 31 RETURNING a, b;

UPDATE main_view SET b = 0 WHERE false;

DELETE FROM main_view WHERE a IN (20,21);

DELETE FROM main_view WHERE a = 31 RETURNING a, b;

DROP TRIGGER instead_of_insert_trig ON main_view;

DROP TRIGGER instead_of_delete_trig ON main_view;

DROP VIEW main_view;

CREATE TABLE country_table (
    country_id        serial primary key,
    country_name    text unique not null,
    continent        text not null
);

INSERT INTO country_table (country_name, continent)
    VALUES ('Japan', 'Asia'),
           ('UK', 'Europe'),
           ('USA', 'North America')
    RETURNING *;

CREATE TABLE city_table (
    city_id        serial primary key,
    city_name    text not null,
    population    bigint,
    country_id    int references country_table
);

CREATE VIEW city_view AS
    SELECT city_id, city_name, population, country_name, continent
    FROM city_table ci
    LEFT JOIN country_table co ON co.country_id = ci.country_id;

CREATE FUNCTION city_insert() RETURNS trigger LANGUAGE plpgsql AS $$
declare
    ctry_id int;
begin
    if NEW.country_name IS NOT NULL then
        SELECT country_id, continent INTO ctry_id, NEW.continent
            FROM country_table WHERE country_name = NEW.country_name;
        if NOT FOUND then
            raise exception 'No such country: "%"', NEW.country_name;
        end if;
    else
        NEW.continent := NULL;
    end if;

    if NEW.city_id IS NOT NULL then
        INSERT INTO city_table
            VALUES(NEW.city_id, NEW.city_name, NEW.population, ctry_id);
    else
        INSERT INTO city_table(city_name, population, country_id)
            VALUES(NEW.city_name, NEW.population, ctry_id)
            RETURNING city_id INTO NEW.city_id;
    end if;

    RETURN NEW;
end;
$$;

CREATE TRIGGER city_insert_trig INSTEAD OF INSERT ON city_view
FOR EACH ROW EXECUTE PROCEDURE city_insert();

CREATE FUNCTION city_delete() RETURNS trigger LANGUAGE plpgsql AS $$
begin
    DELETE FROM city_table WHERE city_id = OLD.city_id;
    if NOT FOUND then RETURN NULL; end if;
    RETURN OLD;
end;
$$;

CREATE TRIGGER city_delete_trig INSTEAD OF DELETE ON city_view
FOR EACH ROW EXECUTE PROCEDURE city_delete();

CREATE FUNCTION city_update() RETURNS trigger LANGUAGE plpgsql AS $$
declare
    ctry_id int;
begin
    if NEW.country_name IS DISTINCT FROM OLD.country_name then
        SELECT country_id, continent INTO ctry_id, NEW.continent
            FROM country_table WHERE country_name = NEW.country_name;
        if NOT FOUND then
            raise exception 'No such country: "%"', NEW.country_name;
        end if;

        UPDATE city_table SET city_name = NEW.city_name,
                              population = NEW.population,
                              country_id = ctry_id
            WHERE city_id = OLD.city_id;
    else
        UPDATE city_table SET city_name = NEW.city_name,
                              population = NEW.population
            WHERE city_id = OLD.city_id;
        NEW.continent := OLD.continent;
    end if;

    if NOT FOUND then RETURN NULL; end if;
    RETURN NEW;
end;
$$;

CREATE TRIGGER city_update_trig INSTEAD OF UPDATE ON city_view
FOR EACH ROW EXECUTE PROCEDURE city_update();

INSERT INTO city_view(city_name) VALUES('Tokyo') RETURNING *;

INSERT INTO city_view(city_name, population) VALUES('London', 7556900) RETURNING *;

INSERT INTO city_view(city_name, country_name) VALUES('Washington DC', 'USA') RETURNING *;

INSERT INTO city_view(city_id, city_name) VALUES(123456, 'New York') RETURNING *;

INSERT INTO city_view VALUES(234567, 'Birmingham', 1016800, 'UK', 'EU') RETURNING *;

UPDATE city_view SET country_name = 'Japon' WHERE city_name = 'Tokyo';

UPDATE city_view SET country_name = 'Japan' WHERE city_name = 'Takyo';

UPDATE city_view SET country_name = 'Japan' WHERE city_name = 'Tokyo' RETURNING *;

UPDATE city_view SET population = 13010279 WHERE city_name = 'Tokyo' RETURNING *;

UPDATE city_view SET country_name = 'UK' WHERE city_name = 'New York' RETURNING *;

UPDATE city_view SET country_name = 'USA', population = 8391881 WHERE city_name = 'New York' RETURNING *;

UPDATE city_view SET continent = 'EU' WHERE continent = 'Europe' RETURNING *;

UPDATE city_view v1 SET country_name = v2.country_name FROM city_view v2
    WHERE v2.city_name = 'Birmingham' AND v1.city_name = 'London' RETURNING *;

DELETE FROM city_view WHERE city_name = 'Birmingham' RETURNING *;

CREATE VIEW european_city_view AS
    SELECT * FROM city_view WHERE continent = 'Europe';

SELECT count(*) FROM european_city_view;

CREATE FUNCTION no_op_trig_fn() RETURNS trigger LANGUAGE plpgsql
AS 'begin RETURN NULL; end';

CREATE TRIGGER no_op_trig INSTEAD OF INSERT OR UPDATE OR DELETE
ON european_city_view FOR EACH ROW EXECUTE PROCEDURE no_op_trig_fn();

INSERT INTO european_city_view VALUES (0, 'x', 10000, 'y', 'z');

UPDATE european_city_view SET population = 10000;

DELETE FROM european_city_view;

CREATE RULE european_city_insert_rule AS ON INSERT TO european_city_view
DO INSTEAD INSERT INTO city_view
VALUES (NEW.city_id, NEW.city_name, NEW.population, NEW.country_name, NEW.continent)
RETURNING *;

CREATE RULE european_city_update_rule AS ON UPDATE TO european_city_view
DO INSTEAD UPDATE city_view SET
    city_name = NEW.city_name,
    population = NEW.population,
    country_name = NEW.country_name
WHERE city_id = OLD.city_id
RETURNING NEW.*;

CREATE RULE european_city_delete_rule AS ON DELETE TO european_city_view
DO INSTEAD DELETE FROM city_view WHERE city_id = OLD.city_id RETURNING *;

INSERT INTO european_city_view(city_name, country_name)
    VALUES ('Cambridge', 'USA') RETURNING *;

UPDATE european_city_view SET country_name = 'UK'
    WHERE city_name = 'Cambridge';

DELETE FROM european_city_view WHERE city_name = 'Cambridge';

UPDATE city_view SET country_name = 'UK'
    WHERE city_name = 'Cambridge' RETURNING *;

UPDATE european_city_view SET population = 122800
    WHERE city_name = 'Cambridge' RETURNING *;

DELETE FROM european_city_view WHERE city_name = 'Cambridge' RETURNING *;

UPDATE city_view v SET population = 599657
    FROM city_table ci, country_table co
    WHERE ci.city_name = 'Washington DC' and co.country_name = 'USA'
    AND v.city_id = ci.city_id AND v.country_name = co.country_name
    RETURNING co.country_id, v.country_name,
              v.city_id, v.city_name, v.population;

SELECT * FROM city_view;

DROP TABLE city_table CASCADE;

DROP TABLE country_table;

create table depth_a (id int not null primary key);

create table depth_b (id int not null primary key);

create table depth_c (id int not null primary key);

create function depth_a_tf() returns trigger
  language plpgsql as $$
begin
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  insert into depth_b values (new.id);
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  return new;
end;
$$;

create trigger depth_a_tr before insert on depth_a
  for each row execute procedure depth_a_tf();

create function depth_b_tf() returns trigger
  language plpgsql as $$
begin
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  begin
    execute 'insert into depth_c values (' || new.id::text || ')';
  exception
    when sqlstate 'U9999' then
      raise notice 'SQLSTATE = U9999: depth = %', pg_trigger_depth();
  end;
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  if new.id = 1 then
    execute 'insert into depth_c values (' || new.id::text || ')';
  end if;
  return new;
end;
$$;

create trigger depth_b_tr before insert on depth_b
  for each row execute procedure depth_b_tf();

create function depth_c_tf() returns trigger
  language plpgsql as $$
begin
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  if new.id = 1 then
    raise exception sqlstate 'U9999';
  end if;
  raise notice '%: depth = %', tg_name, pg_trigger_depth();
  return new;
end;
$$;

create trigger depth_c_tr before insert on depth_c
  for each row execute procedure depth_c_tf();

select pg_trigger_depth();

insert into depth_a values (1);

select pg_trigger_depth();

insert into depth_a values (2);

select pg_trigger_depth();

drop table depth_a, depth_b, depth_c;

drop function depth_a_tf();

drop function depth_b_tf();

drop function depth_c_tf();

create temp table parent (
    aid int not null primary key,
    val1 text,
    val2 text,
    val3 text,
    val4 text,
    bcnt int not null default 0);

create temp table child (
    bid int not null primary key,
    aid int not null,
    val1 text);

create function parent_upd_func()
  returns trigger language plpgsql as
$$
begin
  if old.val1 <> new.val1 then
    new.val2 = new.val1;
    delete from child where child.aid = new.aid and child.val1 = new.val1;
  end if;
  return new;
end;
$$;

create trigger parent_upd_trig before update on parent
  for each row execute procedure parent_upd_func();

create function parent_del_func()
  returns trigger language plpgsql as
$$
begin
  delete from child where aid = old.aid;
  return old;
end;
$$;

create trigger parent_del_trig before delete on parent
  for each row execute procedure parent_del_func();

create function child_ins_func()
  returns trigger language plpgsql as
$$
begin
  update parent set bcnt = bcnt + 1 where aid = new.aid;
  return new;
end;
$$;

create trigger child_ins_trig after insert on child
  for each row execute procedure child_ins_func();

create function child_del_func()
  returns trigger language plpgsql as
$$
begin
  update parent set bcnt = bcnt - 1 where aid = old.aid;
  return old;
end;
$$;

create trigger child_del_trig after delete on child
  for each row execute procedure child_del_func();

insert into parent values (1, 'a', 'a', 'a', 'a', 0);

insert into child values (10, 1, 'b');

select * from parent;

select * from child;

update parent set val1 = 'b' where aid = 1;

select * from parent;

select * from child;

delete from parent where aid = 1;

select * from parent;

select * from child;

create or replace function parent_del_func()
  returns trigger language plpgsql as
$$
begin
  delete from child where aid = old.aid;
  if found then
    delete from parent where aid = old.aid;
    return null; -- cancel outer deletion
  end if;
  return old;
end;
$$;

delete from parent where aid = 1;

select * from parent;

select * from child;

drop table parent, child;

drop function parent_upd_func();

drop function parent_del_func();

drop function child_ins_func();

drop function child_del_func();

create temp table self_ref_trigger (
    id int primary key,
    parent int references self_ref_trigger,
    data text,
    nchildren int not null default 0
);

create function self_ref_trigger_ins_func()
  returns trigger language plpgsql as
$$
begin
  if new.parent is not null then
    update self_ref_trigger set nchildren = nchildren + 1
      where id = new.parent;
  end if;
  return new;
end;
$$;

create trigger self_ref_trigger_ins_trig before insert on self_ref_trigger
  for each row execute procedure self_ref_trigger_ins_func();

create function self_ref_trigger_del_func()
  returns trigger language plpgsql as
$$
begin
  if old.parent is not null then
    update self_ref_trigger set nchildren = nchildren - 1
      where id = old.parent;
  end if;
  return old;
end;
$$;

create trigger self_ref_trigger_del_trig before delete on self_ref_trigger
  for each row execute procedure self_ref_trigger_del_func();

insert into self_ref_trigger values (1, null, 'root');

insert into self_ref_trigger values (2, 1, 'root child A');

insert into self_ref_trigger values (3, 1, 'root child B');

insert into self_ref_trigger values (4, 2, 'grandchild 1');

insert into self_ref_trigger values (5, 3, 'grandchild 2');

update self_ref_trigger set data = 'root!' where id = 1;

select * from self_ref_trigger;

delete from self_ref_trigger;

select * from self_ref_trigger;

drop table self_ref_trigger;

drop function self_ref_trigger_ins_func();

drop function self_ref_trigger_del_func();

create table stmt_trig_on_empty_upd (a int);

create table stmt_trig_on_empty_upd1 () inherits (stmt_trig_on_empty_upd);

create function update_stmt_notice() returns trigger as $$
begin
	raise notice 'updating %', TG_TABLE_NAME;
	return null;
end;
$$ language plpgsql;

create trigger before_stmt_trigger
	before update on stmt_trig_on_empty_upd
	execute procedure update_stmt_notice();

create trigger before_stmt_trigger
	before update on stmt_trig_on_empty_upd1
	execute procedure update_stmt_notice();

update stmt_trig_on_empty_upd set a = a where false returning a+1 as aa;

update stmt_trig_on_empty_upd1 set a = a where false returning a+1 as aa;

drop table stmt_trig_on_empty_upd cascade;

drop function update_stmt_notice();

create table trigger_ddl_table (
   col1 integer,
   col2 integer
);

create function trigger_ddl_func() returns trigger as $$
begin
  alter table trigger_ddl_table add primary key (col1);
  return new;
end$$ language plpgsql;

create trigger trigger_ddl_func before insert on trigger_ddl_table for each row
  execute procedure trigger_ddl_func();

insert into trigger_ddl_table values (1, 42);

create or replace function trigger_ddl_func() returns trigger as $$
begin
  create index on trigger_ddl_table (col2);
  return new;
end$$ language plpgsql;

insert into trigger_ddl_table values (1, 42);

drop table trigger_ddl_table;

drop function trigger_ddl_func();

create table upsert (key int4 primary key, color text);

create function upsert_before_func()
  returns trigger language plpgsql as
$$
begin
  if (TG_OP = 'UPDATE') then
    raise warning 'before update (old): %', old.*::text;
    raise warning 'before update (new): %', new.*::text;
  elsif (TG_OP = 'INSERT') then
    raise warning 'before insert (new): %', new.*::text;
    if new.key % 2 = 0 then
      new.key := new.key + 1;
      new.color := new.color || ' trig modified';
      raise warning 'before insert (new, modified): %', new.*::text;
    end if;
  end if;
  return new;
end;
$$;

create trigger upsert_before_trig before insert or update on upsert
  for each row execute procedure upsert_before_func();

create function upsert_after_func()
  returns trigger language plpgsql as
$$
begin
  if (TG_OP = 'UPDATE') then
    raise warning 'after update (old): %', old.*::text;
    raise warning 'after update (new): %', new.*::text;
  elsif (TG_OP = 'INSERT') then
    raise warning 'after insert (new): %', new.*::text;
  end if;
  return null;
end;
$$;

create trigger upsert_after_trig after insert or update on upsert
  for each row execute procedure upsert_after_func();

insert into upsert values(1, 'black') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(2, 'red') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(3, 'orange') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(4, 'green') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(5, 'purple') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(6, 'white') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(7, 'pink') on conflict (key) do update set color = 'updated ' || upsert.color;

insert into upsert values(8, 'yellow') on conflict (key) do update set color = 'updated ' || upsert.color;

select * from upsert;

drop table upsert;

drop function upsert_before_func();

drop function upsert_after_func();

create table my_table (i int);

create view my_view as select * from my_table;

create function my_trigger_function() returns trigger as $$ begin end; $$ language plpgsql;

create trigger my_trigger after update on my_view referencing old table as old_table
   for each statement execute procedure my_trigger_function();

drop function my_trigger_function();

drop view my_view;

drop table my_table;

create table parted_trig (a int) partition by list (a);

create function trigger_nothing() returns trigger
  language plpgsql as $$ begin end; $$;

create trigger failed instead of update on parted_trig
  for each row execute procedure trigger_nothing();

create trigger failed after update on parted_trig
  referencing old table as old_table
  for each row execute procedure trigger_nothing();

drop table parted_trig;

create table trigpart (a int, b int) partition by range (a);

create table trigpart1 partition of trigpart for values from (0) to (1000);

create trigger trg1 after insert on trigpart for each row execute procedure trigger_nothing();

create table trigpart2 partition of trigpart for values from (1000) to (2000);

create table trigpart3 (like trigpart);

alter table trigpart attach partition trigpart3 for values from (2000) to (3000);

create table trigpart4 partition of trigpart for values from (3000) to (4000) partition by range (a);

create table trigpart41 partition of trigpart4 for values from (3000) to (3500);

create table trigpart42 (like trigpart);

alter table trigpart4 attach partition trigpart42 for values from (3500) to (4000);

select tgrelid::regclass, tgname, tgfoid::regproc from pg_trigger
  where tgrelid::regclass::text like 'trigpart%' order by tgrelid::regclass::text;

drop trigger trg1 on trigpart1;

drop trigger trg1 on trigpart2;

drop trigger trg1 on trigpart3;

drop table trigpart2;

select tgrelid::regclass, tgname, tgfoid::regproc from pg_trigger
  where tgrelid::regclass::text like 'trigpart%' order by tgrelid::regclass::text;

drop trigger trg1 on trigpart;

select tgrelid::regclass, tgname, tgfoid::regproc from pg_trigger
  where tgrelid::regclass::text like 'trigpart%' order by tgrelid::regclass::text;

create trigger trg1 after insert on trigpart for each row execute procedure trigger_nothing();

alter table trigpart detach partition trigpart3;

drop trigger trg1 on trigpart3;

alter table trigpart detach partition trigpart4;

drop trigger trg1 on trigpart41;

drop table trigpart4;

alter table trigpart attach partition trigpart3 for values from (2000) to (3000);

alter table trigpart detach partition trigpart3;

alter table trigpart attach partition trigpart3 for values from (2000) to (3000);

drop table trigpart3;

select tgrelid::regclass::text, tgname, tgfoid::regproc, tgenabled, tgisinternal from pg_trigger
  where tgname ~ '^trg1' order by 1;

create table trigpart3 (like trigpart);

create trigger trg1 after insert on trigpart3 for each row execute procedure trigger_nothing();

alter table trigpart attach partition trigpart3 FOR VALUES FROM (2000) to (3000);

drop table trigpart3;

create trigger samename after delete on trigpart execute function trigger_nothing();

create trigger samename after delete on trigpart1 execute function trigger_nothing();

drop table trigpart;

drop function trigger_nothing();

create table parted_stmt_trig (a int) partition by list (a);

create table parted_stmt_trig1 partition of parted_stmt_trig for values in (1);

create table parted_stmt_trig2 partition of parted_stmt_trig for values in (2);

create table parted2_stmt_trig (a int) partition by list (a);

create table parted2_stmt_trig1 partition of parted2_stmt_trig for values in (1);

create table parted2_stmt_trig2 partition of parted2_stmt_trig for values in (2);

create or replace function trigger_notice() returns trigger as $$
  begin
    raise notice 'trigger % on % % % for %', TG_NAME, TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL;
    if TG_LEVEL = 'ROW' then
       return NEW;
    end if;
    return null;
  end;
  $$ language plpgsql;

create trigger trig_ins_before before insert on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_ins_after after insert on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_upd_before before update on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_upd_after after update on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_del_before before delete on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_del_after after delete on parted_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_ins_after_parent after insert on parted_stmt_trig
  for each row execute procedure trigger_notice();

create trigger trig_upd_after_parent after update on parted_stmt_trig
  for each row execute procedure trigger_notice();

create trigger trig_del_after_parent after delete on parted_stmt_trig
  for each row execute procedure trigger_notice();

create trigger trig_ins_before_child before insert on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_ins_after_child after insert on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_upd_before_child before update on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_upd_after_child after update on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_del_before_child before delete on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_del_after_child after delete on parted_stmt_trig1
  for each row execute procedure trigger_notice();

create trigger trig_ins_before_3 before insert on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_ins_after_3 after insert on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_upd_before_3 before update on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_upd_after_3 after update on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_del_before_3 before delete on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

create trigger trig_del_after_3 after delete on parted2_stmt_trig
  for each statement execute procedure trigger_notice();

with ins (a) as (
  insert into parted2_stmt_trig values (1), (2) returning a
) insert into parted_stmt_trig select a from ins returning tableoid::regclass, a;

with upd as (
  update parted2_stmt_trig set a = a
) update parted_stmt_trig  set a = a;

delete from parted_stmt_trig;

copy parted_stmt_trig(a) from stdin;

copy parted_stmt_trig1(a) from stdin;

alter table parted_stmt_trig disable trigger trig_ins_after_parent;

insert into parted_stmt_trig values (1);

alter table parted_stmt_trig enable trigger trig_ins_after_parent;

insert into parted_stmt_trig values (1);

drop table parted_stmt_trig, parted2_stmt_trig;

create table parted_trig (a int) partition by range (a);

create table parted_trig_1 partition of parted_trig for values from (0) to (1000)
   partition by range (a);

create table parted_trig_1_1 partition of parted_trig_1 for values from (0) to (100);

create table parted_trig_2 partition of parted_trig for values from (1000) to (2000);

create trigger zzz after insert on parted_trig for each row execute procedure trigger_notice();

create trigger mmm after insert on parted_trig_1_1 for each row execute procedure trigger_notice();

create trigger aaa after insert on parted_trig_1 for each row execute procedure trigger_notice();

create trigger bbb after insert on parted_trig for each row execute procedure trigger_notice();

create trigger qqq after insert on parted_trig_1_1 for each row execute procedure trigger_notice();

insert into parted_trig values (50), (1500);

drop table parted_trig;

create table parted_trig (a int) partition by list (a);

create table parted_trig1 partition of parted_trig for values in (1);

create table parted_trig2 partition of parted_trig for values in (2);

insert into parted_trig values (1);

create or replace function trigger_notice() returns trigger as $$
  begin
    raise notice 'trigger % on % % % for %', TG_NAME, TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL;
    if TG_LEVEL = 'ROW' then
      if TG_OP = 'DELETE' then
        return OLD;
      else
        return NEW;
      end if;
    end if;
    return null;
  end;
  $$ language plpgsql;

create trigger parted_trig_before_stmt before insert or update or delete on parted_trig
   for each statement execute procedure trigger_notice();

create trigger parted_trig_before_row before insert or update or delete on parted_trig
   for each row execute procedure trigger_notice();

create trigger parted_trig_after_row after insert or update or delete on parted_trig
   for each row execute procedure trigger_notice();

create trigger parted_trig_after_stmt after insert or update or delete on parted_trig
   for each statement execute procedure trigger_notice();

update parted_trig set a = 2 where a = 1;

drop table parted_trig;

create table parted_trig (a int) partition by list (a);

create table parted_trig1 partition of parted_trig for values in (1);

create or replace function trigger_notice() returns trigger as $$
  declare
    arg1 text = TG_ARGV[0];
    arg2 integer = TG_ARGV[1];
  begin
    raise notice 'trigger % on % % % for % args % %',
		TG_NAME, TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL, arg1, arg2;
    return null;
  end;
  $$ language plpgsql;

create trigger aaa after insert on parted_trig
   for each row execute procedure trigger_notice('quirky', 1);

create table parted_trig2 partition of parted_trig for values in (2);

create table parted_trig3 (like parted_trig);

alter table parted_trig attach partition parted_trig3 for values in (3);

insert into parted_trig values (1), (2), (3);

drop table parted_trig;

create function bark(text) returns bool language plpgsql immutable
  as $$ begin raise notice '% <- woof!', $1; return true; end; $$;

create or replace function trigger_notice_ab() returns trigger as $$
  begin
    raise notice 'trigger % on % % % for %: (a,b)=(%,%)',
		TG_NAME, TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL,
		NEW.a, NEW.b;
    if TG_LEVEL = 'ROW' then
       return NEW;
    end if;
    return null;
  end;
  $$ language plpgsql;

create table parted_irreg_ancestor (fd text, b text, fd2 int, fd3 int, a int)
  partition by range (b);

alter table parted_irreg_ancestor drop column fd,
  drop column fd2, drop column fd3;

create table parted_irreg (fd int, a int, fd2 int, b text)
  partition by range (b);

alter table parted_irreg drop column fd, drop column fd2;

alter table parted_irreg_ancestor attach partition parted_irreg
  for values from ('aaaa') to ('zzzz');

create table parted1_irreg (b text, fd int, a int);

alter table parted1_irreg drop column fd;

alter table parted_irreg attach partition parted1_irreg
  for values from ('aaaa') to ('bbbb');

create trigger parted_trig after insert on parted_irreg
  for each row execute procedure trigger_notice_ab();

create trigger parted_trig_odd after insert on parted_irreg for each row
  when (bark(new.b) AND new.a % 2 = 1) execute procedure trigger_notice_ab();

insert into parted_irreg values (1, 'aardvark'), (2, 'aanimals');

insert into parted1_irreg values ('aardwolf', 2);

insert into parted_irreg_ancestor values ('aasvogel', 3);

drop table parted_irreg_ancestor;

create table parted (a int, b int, c text) partition by list (a);

create table parted_1 partition of parted for values in (1)
  partition by list (b);

create table parted_1_1 partition of parted_1 for values in (1);

create function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.a = new.a + 1;
  return new;
end;
$$;

insert into parted values (1, 1, 'uno uno v1');

create trigger t before insert or update or delete on parted
  for each row execute function parted_trigfunc();

insert into parted values (1, 1, 'uno uno v2');

update parted set c = c || 'v3';

create or replace function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.b = new.b + 1;
  return new;
end;
$$;

insert into parted values (1, 1, 'uno uno v4');

update parted set c = c || 'v5';

create or replace function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.c = new.c || ' did '|| TG_OP;
  return new;
end;
$$;

insert into parted values (1, 1, 'uno uno');

update parted set c = c || ' v6';

select tableoid::regclass, * from parted;

truncate table parted;

create table parted_2 partition of parted for values in (2);

insert into parted values (1, 1, 'uno uno v5');

update parted set a = 2;

select tableoid::regclass, * from parted;

create or replace function parted_trigfunc2() returns trigger language plpgsql as $$
begin
  new.a = new.a + 1;
  return new;
end;
$$;

create trigger t2 before update on parted
  for each row execute function parted_trigfunc2();

truncate table parted;

insert into parted values (1, 1, 'uno uno v6');

create table parted_3 partition of parted for values in (3);

update parted set a = a + 1;

select tableoid::regclass, * from parted;

update parted set a = 0;

select tableoid::regclass, * from parted;

drop table parted;

create table parted (a int, b int, c text) partition by list ((a + b));

create or replace function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.a = new.a + new.b;
  return new;
end;
$$;

create table parted_1 partition of parted for values in (1, 2);

create table parted_2 partition of parted for values in (3, 4);

create trigger t before insert or update on parted
  for each row execute function parted_trigfunc();

insert into parted values (0, 1, 'zero win');

insert into parted values (1, 1, 'one fail');

insert into parted values (1, 2, 'two fail');

select * from parted;

drop table parted;

drop function parted_trigfunc();

create table parted_constr_ancestor (a int, b text)
  partition by range (b);

create table parted_constr (a int, b text)
  partition by range (b);

alter table parted_constr_ancestor attach partition parted_constr
  for values from ('aaaa') to ('zzzz');

create table parted1_constr (a int, b text);

alter table parted_constr attach partition parted1_constr
  for values from ('aaaa') to ('bbbb');

create constraint trigger parted_trig after insert on parted_constr_ancestor
  deferrable
  for each row execute procedure trigger_notice_ab();

begin;

insert into parted_constr values (1, 'aardvark');

insert into parted1_constr values (2, 'aardwolf');

insert into parted_constr_ancestor values (3, 'aasvogel');

commit;

begin;

set constraints parted_trig deferred;

insert into parted_constr values (1, 'aardvark');

insert into parted1_constr values (2, 'aardwolf'), (3, 'aasvogel');

commit;

drop table parted_constr_ancestor;

drop function bark(text);

create table parted_trigger (a int, b text) partition by range (a);

create table parted_trigger_1 partition of parted_trigger for values from (0) to (1000);

create table parted_trigger_2 (drp int, a int, b text);

alter table parted_trigger_2 drop column drp;

alter table parted_trigger attach partition parted_trigger_2 for values from (1000) to (2000);

create trigger parted_trigger after update on parted_trigger
  for each row when (new.a % 2 = 1 and length(old.b) >= 2) execute procedure trigger_notice_ab();

create table parted_trigger_3 (b text, a int) partition by range (length(b));

create table parted_trigger_3_1 partition of parted_trigger_3 for values from (1) to (3);

create table parted_trigger_3_2 partition of parted_trigger_3 for values from (3) to (5);

alter table parted_trigger attach partition parted_trigger_3 for values from (2000) to (3000);

insert into parted_trigger values
    (0, 'a'), (1, 'bbb'), (2, 'bcd'), (3, 'c'),
	(1000, 'c'), (1001, 'ddd'), (1002, 'efg'), (1003, 'f'),
	(2000, 'e'), (2001, 'fff'), (2002, 'ghi'), (2003, 'h');

update parted_trigger set a = a + 2;

drop table parted_trigger;

create table parted_referenced (a int);

create table unparted_trigger (a int, b text);

create table parted_trigger (a int, b text) partition by range (a);

create table parted_trigger_1 partition of parted_trigger for values from (0) to (1000);

create table parted_trigger_2 (drp int, a int, b text);

alter table parted_trigger_2 drop column drp;

alter table parted_trigger attach partition parted_trigger_2 for values from (1000) to (2000);

create constraint trigger parted_trigger after update on parted_trigger
  from parted_referenced
  for each row execute procedure trigger_notice_ab();

create constraint trigger parted_trigger after update on unparted_trigger
  from parted_referenced
  for each row execute procedure trigger_notice_ab();

create table parted_trigger_3 (b text, a int) partition by range (length(b));

create table parted_trigger_3_1 partition of parted_trigger_3 for values from (1) to (3);

create table parted_trigger_3_2 partition of parted_trigger_3 for values from (3) to (5);

alter table parted_trigger attach partition parted_trigger_3 for values from (2000) to (3000);

select tgname, conname, t.tgrelid::regclass, t.tgconstrrelid::regclass,
  c.conrelid::regclass, c.confrelid::regclass
  from pg_trigger t join pg_constraint c on (t.tgconstraint = c.oid)
  where tgname = 'parted_trigger'
  order by t.tgrelid::regclass::text;

drop table parted_referenced, parted_trigger, unparted_trigger;

create table parted_trigger (a int, b text) partition by range (a);

create table parted_trigger_1 partition of parted_trigger for values from (0) to (1000);

create table parted_trigger_2 (drp int, a int, b text);

alter table parted_trigger_2 drop column drp;

alter table parted_trigger attach partition parted_trigger_2 for values from (1000) to (2000);

create trigger parted_trigger after update of b on parted_trigger
  for each row execute procedure trigger_notice_ab();

create table parted_trigger_3 (b text, a int) partition by range (length(b));

create table parted_trigger_3_1 partition of parted_trigger_3 for values from (1) to (4);

create table parted_trigger_3_2 partition of parted_trigger_3 for values from (4) to (8);

alter table parted_trigger attach partition parted_trigger_3 for values from (2000) to (3000);

insert into parted_trigger values (0, 'a'), (1000, 'c'), (2000, 'e'), (2001, 'eeee');

update parted_trigger set a = a + 2;

update parted_trigger set b = b || 'b';

drop table parted_trigger;

drop function trigger_notice_ab();

create table trg_clone (a int) partition by range (a);

create table trg_clone1 partition of trg_clone for values from (0) to (1000);

alter table trg_clone add constraint uniq unique (a) deferrable;

create table trg_clone2 partition of trg_clone for values from (1000) to (2000);

create table trg_clone3 partition of trg_clone for values from (2000) to (3000)
  partition by range (a);

create table trg_clone_3_3 partition of trg_clone3 for values from (2000) to (2100);

select tgrelid::regclass, count(*) from pg_trigger
  where tgrelid::regclass in ('trg_clone', 'trg_clone1', 'trg_clone2',
	'trg_clone3', 'trg_clone_3_3')
  group by tgrelid::regclass order by tgrelid::regclass;

drop table trg_clone;

create table parent (a int);

create table child1 () inherits (parent);

create function trig_nothing() returns trigger language plpgsql
  as $$ begin return null; end $$;

create trigger tg after insert on parent
  for each row execute function trig_nothing();

create trigger tg after insert on child1
  for each row execute function trig_nothing();

alter table parent disable trigger tg;

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text;

alter table only parent enable always trigger tg;

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text;

drop table parent, child1;

create table parent (a int) partition by list (a);

create table child1 partition of parent for values in (1);

create trigger tg after insert on parent
  for each row execute procedure trig_nothing();

create trigger tg_stmt after insert on parent
  for statement execute procedure trig_nothing();

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgname;

alter table only parent enable always trigger tg;

alter table parent enable always trigger tg_stmt;

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgname;

alter table parent enable always trigger tg;

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgname;

alter table parent disable trigger user;

select tgrelid::regclass, tgname, tgenabled from pg_trigger
  where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgname;

drop table parent, child1;

create table parent (a int primary key, f int references parent)
  partition by list (a);

create table child1 partition of parent for values in (1);

select tgrelid::regclass, rtrim(tgname, '0123456789') as tgname,
  tgfoid::regproc, tgenabled
  from pg_trigger where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgfoid;

alter table parent disable trigger all;

select tgrelid::regclass, rtrim(tgname, '0123456789') as tgname,
  tgfoid::regproc, tgenabled
  from pg_trigger where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgfoid;

drop table parent, child1;

CREATE TABLE trgfire (i int) PARTITION BY RANGE (i);

CREATE TABLE trgfire1 PARTITION OF trgfire FOR VALUES FROM (1) TO (10);

CREATE OR REPLACE FUNCTION tgf() RETURNS trigger LANGUAGE plpgsql
  AS $$ begin raise exception 'except'; end $$;

CREATE TRIGGER tg AFTER INSERT ON trgfire FOR EACH ROW EXECUTE FUNCTION tgf();

INSERT INTO trgfire VALUES (1);

ALTER TABLE trgfire DISABLE TRIGGER tg;

INSERT INTO trgfire VALUES (1);

CREATE TABLE trgfire2 PARTITION OF trgfire FOR VALUES FROM (10) TO (20);

INSERT INTO trgfire VALUES (11);

CREATE TABLE trgfire3 (LIKE trgfire);

ALTER TABLE trgfire ATTACH PARTITION trgfire3 FOR VALUES FROM (20) TO (30);

INSERT INTO trgfire VALUES (21);

CREATE TABLE trgfire4 PARTITION OF trgfire FOR VALUES FROM (30) TO (40) PARTITION BY LIST (i);

CREATE TABLE trgfire4_30 PARTITION OF trgfire4 FOR VALUES IN (30);

INSERT INTO trgfire VALUES (30);

CREATE TABLE trgfire5 (LIKE trgfire) PARTITION BY LIST (i);

CREATE TABLE trgfire5_40 PARTITION OF trgfire5 FOR VALUES IN (40);

ALTER TABLE trgfire ATTACH PARTITION trgfire5 FOR VALUES FROM (40) TO (50);

INSERT INTO trgfire VALUES (40);

SELECT tgrelid::regclass, tgenabled FROM pg_trigger
  WHERE tgrelid::regclass IN (SELECT oid from pg_class where relname LIKE 'trgfire%')
  ORDER BY tgrelid::regclass::text;

ALTER TABLE trgfire ENABLE TRIGGER tg;

INSERT INTO trgfire VALUES (1);

INSERT INTO trgfire VALUES (11);

INSERT INTO trgfire VALUES (21);

INSERT INTO trgfire VALUES (30);

INSERT INTO trgfire VALUES (40);

DROP TABLE trgfire;

DROP FUNCTION tgf();

create or replace function dump_insert() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, new table = %',
                 TG_NAME,
                 (select string_agg(new_table::text, ', ' order by a) from new_table);
    return null;
  end;
$$;

create or replace function dump_update() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, old table = %, new table = %',
                 TG_NAME,
                 (select string_agg(old_table::text, ', ' order by a) from old_table),
                 (select string_agg(new_table::text, ', ' order by a) from new_table);
    return null;
  end;
$$;

create or replace function dump_delete() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, old table = %',
                 TG_NAME,
                 (select string_agg(old_table::text, ', ' order by a) from old_table);
    return null;
  end;
$$;

create table parent (a text, b int) partition by list (a);

create table child1 partition of parent for values in ('AAA');

create table child2 (x int, a text, b int);

alter table child2 drop column x;

alter table parent attach partition child2 for values in ('BBB');

create table child3 (b int, a text);

alter table parent attach partition child3 for values in ('CCC');

create trigger parent_insert_trig
  after insert on parent referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger parent_update_trig
  after update on parent referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger parent_delete_trig
  after delete on parent referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child1_insert_trig
  after insert on child1 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child1_update_trig
  after update on child1 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child1_delete_trig
  after delete on child1 referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child2_insert_trig
  after insert on child2 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child2_update_trig
  after update on child2 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child2_delete_trig
  after delete on child2 referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child3_insert_trig
  after insert on child3 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child3_update_trig
  after update on child3 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child3_delete_trig
  after delete on child3 referencing old table as old_table
  for each statement execute procedure dump_delete();

SELECT trigger_name, event_manipulation, event_object_schema, event_object_table,
       action_order, action_condition, action_orientation, action_timing,
       action_reference_old_table, action_reference_new_table
  FROM information_schema.triggers
  WHERE event_object_table IN ('parent', 'child1', 'child2', 'child3')
  ORDER BY trigger_name COLLATE "C", 2;

insert into child1 values ('AAA', 42);

insert into child2 values ('BBB', 42);

insert into child3 values (42, 'CCC');

update parent set b = b + 1;

delete from parent;

insert into parent values ('AAA', 42);

insert into parent values ('BBB', 42);

insert into parent values ('CCC', 42);

delete from child1;

delete from child2;

delete from child3;

copy parent (a, b) from stdin;

alter table parent detach partition child1;

alter table parent attach partition child1 for values in ('AAA');

drop trigger child1_insert_trig on child1;

drop trigger child1_update_trig on child1;

drop trigger child1_delete_trig on child1;

drop trigger child2_insert_trig on child2;

drop trigger child2_update_trig on child2;

drop trigger child2_delete_trig on child2;

drop trigger child3_insert_trig on child3;

drop trigger child3_update_trig on child3;

drop trigger child3_delete_trig on child3;

delete from parent;

copy parent (a, b) from stdin;

create or replace function intercept_insert() returns trigger language plpgsql as
$$
  begin
    new.b = new.b + 1000;
    return new;
  end;
$$;

create trigger intercept_insert_child3
  before insert on child3
  for each row execute procedure intercept_insert();

insert into parent values ('AAA', 42), ('BBB', 42), ('CCC', 66);

copy parent (a, b) from stdin;

drop table child1, child2, child3, parent;

drop function intercept_insert();

create table parent (a text, b int) partition by list (a);

create table child partition of parent for values in ('AAA');

create trigger child_row_trig
  after insert on child referencing new table as new_table
  for each row execute procedure dump_insert();

alter table parent detach partition child;

create trigger child_row_trig
  after insert on child referencing new table as new_table
  for each row execute procedure dump_insert();

alter table parent attach partition child for values in ('AAA');

drop trigger child_row_trig on child;

alter table parent attach partition child for values in ('AAA');

drop table child, parent;

create or replace function dump_update_new() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, new table = %', TG_NAME,
                 (select string_agg(new_table::text, ', ' order by a) from new_table);
    return null;
  end;
$$;

create or replace function dump_update_old() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, old table = %', TG_NAME,
                 (select string_agg(old_table::text, ', ' order by a) from old_table);
    return null;
  end;
$$;

create table trans_tab_parent (a text) partition by list (a);

create table trans_tab_child1 partition of trans_tab_parent for values in ('AAA1', 'AAA2');

create table trans_tab_child2 partition of trans_tab_parent for values in ('BBB1', 'BBB2');

create trigger trans_tab_parent_update_trig
  after update on trans_tab_parent referencing old table as old_table
  for each statement execute procedure dump_update_old();

create trigger trans_tab_parent_insert_trig
  after insert on trans_tab_parent referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger trans_tab_parent_delete_trig
  after delete on trans_tab_parent referencing old table as old_table
  for each statement execute procedure dump_delete();

insert into trans_tab_parent values ('AAA1'), ('BBB1');

update trans_tab_parent set a = 'BBB2' where a = 'AAA1';

drop trigger trans_tab_parent_update_trig on trans_tab_parent;

create trigger trans_tab_parent_update_trig
  after update on trans_tab_parent referencing new table as new_table
  for each statement execute procedure dump_update_new();

update trans_tab_parent set a = 'AAA2' where a = 'BBB1';

delete from trans_tab_parent;

drop table trans_tab_parent, trans_tab_child1, trans_tab_child2;

drop function dump_update_new, dump_update_old;

create table parent (a text, b int);

create table child1 () inherits (parent);

create table child2 (b int, a text);

alter table child2 inherit parent;

create table child3 (c text) inherits (parent);

create trigger parent_insert_trig
  after insert on parent referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger parent_update_trig
  after update on parent referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger parent_delete_trig
  after delete on parent referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child1_insert_trig
  after insert on child1 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child1_update_trig
  after update on child1 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child1_delete_trig
  after delete on child1 referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child2_insert_trig
  after insert on child2 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child2_update_trig
  after update on child2 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child2_delete_trig
  after delete on child2 referencing old table as old_table
  for each statement execute procedure dump_delete();

create trigger child3_insert_trig
  after insert on child3 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger child3_update_trig
  after update on child3 referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger child3_delete_trig
  after delete on child3 referencing old table as old_table
  for each statement execute procedure dump_delete();

insert into child1 values ('AAA', 42);

insert into child2 values (42, 'BBB');

insert into child3 values ('CCC', 42, 'foo');

update parent set b = b + 1;

delete from parent;

insert into child1 values ('AAA', 42);

insert into child2 values (42, 'BBB');

insert into child3 values ('CCC', 42, 'foo');

delete from child1;

delete from child2;

delete from child3;

copy parent (a, b) from stdin;

create index on parent(b);

copy parent (a, b) from stdin;

alter table child1 no inherit parent;

alter table child1 inherit parent;

drop trigger child1_insert_trig on child1;

drop trigger child1_update_trig on child1;

drop trigger child1_delete_trig on child1;

drop trigger child2_insert_trig on child2;

drop trigger child2_update_trig on child2;

drop trigger child2_delete_trig on child2;

drop trigger child3_insert_trig on child3;

drop trigger child3_update_trig on child3;

drop trigger child3_delete_trig on child3;

delete from parent;

drop table child1, child2, child3, parent;

create table parent (a text, b int);

create table child () inherits (parent);

create trigger child_row_trig
  after insert on child referencing new table as new_table
  for each row execute procedure dump_insert();

alter table child no inherit parent;

create trigger child_row_trig
  after insert on child referencing new table as new_table
  for each row execute procedure dump_insert();

alter table child inherit parent;

drop trigger child_row_trig on child;

alter table child inherit parent;

drop table child, parent;

create table table1 (a int);

create table table2 (a text);

create trigger table1_trig
  after insert on table1 referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger table2_trig
  after insert on table2 referencing new table as new_table
  for each statement execute procedure dump_insert();

with wcte as (insert into table1 values (42))
  insert into table2 values ('hello world');

with wcte as (insert into table1 values (43))
  insert into table1 values (44);

select * from table1;

select * from table2;

drop table table1;

drop table table2;

create table my_table (a int primary key, b text);

create trigger my_table_insert_trig
  after insert on my_table referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger my_table_update_trig
  after update on my_table referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

insert into my_table values (1, 'AAA'), (2, 'BBB')
  on conflict (a) do
  update set b = my_table.b || ':' || excluded.b;

insert into my_table values (1, 'AAA'), (2, 'BBB'), (3, 'CCC'), (4, 'DDD')
  on conflict (a) do
  update set b = my_table.b || ':' || excluded.b;

insert into my_table values (3, 'CCC'), (4, 'DDD')
  on conflict (a) do
  update set b = my_table.b || ':' || excluded.b;

create table iocdu_tt_parted (a int primary key, b text) partition by list (a);

create table iocdu_tt_parted1 partition of iocdu_tt_parted for values in (1);

create table iocdu_tt_parted2 partition of iocdu_tt_parted for values in (2);

create table iocdu_tt_parted3 partition of iocdu_tt_parted for values in (3);

create table iocdu_tt_parted4 partition of iocdu_tt_parted for values in (4);

create trigger iocdu_tt_parted_insert_trig
  after insert on iocdu_tt_parted referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger iocdu_tt_parted_update_trig
  after update on iocdu_tt_parted referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

insert into iocdu_tt_parted values (1, 'AAA'), (2, 'BBB')
  on conflict (a) do
  update set b = iocdu_tt_parted.b || ':' || excluded.b;

insert into iocdu_tt_parted values (1, 'AAA'), (2, 'BBB'), (3, 'CCC'), (4, 'DDD')
  on conflict (a) do
  update set b = iocdu_tt_parted.b || ':' || excluded.b;

insert into iocdu_tt_parted values (3, 'CCC'), (4, 'DDD')
  on conflict (a) do
  update set b = iocdu_tt_parted.b || ':' || excluded.b;

drop table iocdu_tt_parted;

create trigger my_table_multievent_trig
  after insert or update on my_table referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger my_table_col_update_trig
  after update of b on my_table referencing new table as new_table
  for each statement execute procedure dump_insert();

drop table my_table;

create table my_table (a int);

create function make_bogus_matview() returns trigger as
$$ begin
  create materialized view transition_test_mv as select * from new_table;
  return new;
end $$
language plpgsql;

create trigger make_bogus_matview
  after insert on my_table
  referencing new table as new_table
  for each statement execute function make_bogus_matview();

insert into my_table values (42);

drop table my_table;

drop function make_bogus_matview();

create table refd_table (a int primary key, b text);

create table trig_table (a int, b text,
  foreign key (a) references refd_table on update cascade on delete cascade
);

create trigger trig_table_before_trig
  before insert or update or delete on trig_table
  for each statement execute procedure trigger_func('trig_table');

create trigger trig_table_insert_trig
  after insert on trig_table referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger trig_table_update_trig
  after update on trig_table referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger trig_table_delete_trig
  after delete on trig_table referencing old table as old_table
  for each statement execute procedure dump_delete();

insert into refd_table values
  (1, 'one'),
  (2, 'two'),
  (3, 'three');

insert into trig_table values
  (1, 'one a'),
  (1, 'one b'),
  (2, 'two a'),
  (2, 'two b'),
  (3, 'three a'),
  (3, 'three b');

update refd_table set a = 11 where b = 'one';

select * from trig_table;

delete from refd_table where length(b) = 3;

select * from trig_table;

drop table refd_table, trig_table;

create table refd_table (id int primary key);

create table trig_table (fk int references refd_table initially deferred);

begin;

insert into trig_table values (1);

drop table refd_table cascade;

commit;

drop table trig_table;

create table self_ref (a int primary key,
                       b int references self_ref(a) on delete cascade);

create trigger self_ref_before_trig
  before delete on self_ref
  for each statement execute procedure trigger_func('self_ref');

create trigger self_ref_r_trig
  after delete on self_ref referencing old table as old_table
  for each row execute procedure dump_delete();

create trigger self_ref_s_trig
  after delete on self_ref referencing old table as old_table
  for each statement execute procedure dump_delete();

insert into self_ref values (1, null), (2, 1), (3, 2);

delete from self_ref where a = 1;

drop trigger self_ref_r_trig on self_ref;

insert into self_ref values (1, null), (2, 1), (3, 2), (4, 3);

delete from self_ref where a = 1;

drop table self_ref;

create table merge_target_table (a int primary key, b text);

create trigger merge_target_table_insert_trig
  after insert on merge_target_table referencing new table as new_table
  for each statement execute procedure dump_insert();

create trigger merge_target_table_update_trig
  after update on merge_target_table referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();

create trigger merge_target_table_delete_trig
  after delete on merge_target_table referencing old table as old_table
  for each statement execute procedure dump_delete();

create table merge_source_table (a int, b text);

insert into merge_source_table
  values (1, 'initial1'), (2, 'initial2'),
		 (3, 'initial3'), (4, 'initial4');

drop table merge_source_table, merge_target_table;

drop function dump_insert();

drop function dump_update();

drop function dump_delete();

create table my_table (id integer);

create function funcA() returns trigger as $$
begin
  raise notice 'hello from funcA';
  return null;
end; $$ language plpgsql;

create function funcB() returns trigger as $$
begin
  raise notice 'hello from funcB';
  return null;
end; $$ language plpgsql;

create trigger my_trig
  after insert on my_table
  for each row execute procedure funcA();

create trigger my_trig
  before insert on my_table
  for each row execute procedure funcB();

insert into my_table values (1);

create or replace trigger my_trig
  before insert on my_table
  for each row execute procedure funcB();

insert into my_table values (2);

table my_table;

drop table my_table;

create table parted_trig (a int) partition by range (a);

create table parted_trig_1 partition of parted_trig
       for values from (0) to (1000) partition by range (a);

create table parted_trig_1_1 partition of parted_trig_1 for values from (0) to (100);

create table parted_trig_2 partition of parted_trig for values from (1000) to (2000);

create table default_parted_trig partition of parted_trig default;

create or replace trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcA();

insert into parted_trig (a) values (50);

create or replace trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcB();

insert into parted_trig (a) values (50);

create or replace trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcA();

insert into parted_trig (a) values (50);

create or replace trigger my_trig
  after insert on parted_trig_1
  for each row execute procedure funcB();

insert into parted_trig (a) values (50);

drop trigger my_trig on parted_trig;

insert into parted_trig (a) values (50);

create trigger my_trig
  after insert on parted_trig_1
  for each row execute procedure funcA();

insert into parted_trig (a) values (50);

create trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcB();

insert into parted_trig (a) values (50);

create or replace trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcB();

insert into parted_trig (a) values (50);

drop table parted_trig;

drop function funcA();

drop function funcB();

create table trigger_parted (a int primary key) partition by list (a);

create function trigger_parted_trigfunc() returns trigger language plpgsql as
  $$ begin end; $$;

create trigger aft_row after insert or update on trigger_parted
  for each row execute function trigger_parted_trigfunc();

create table trigger_parted_p1 partition of trigger_parted for values in (1)
  partition by list (a);

create table trigger_parted_p1_1 partition of trigger_parted_p1 for values in (1);

create table trigger_parted_p2 partition of trigger_parted for values in (2)
  partition by list (a);

create table trigger_parted_p2_2 partition of trigger_parted_p2 for values in (2);

alter table only trigger_parted_p2 disable trigger aft_row;

alter table trigger_parted_p2_2 enable always trigger aft_row;

create table convslot_test_parent (col1 text primary key);

create table convslot_test_child (col1 text primary key,
	foreign key (col1) references convslot_test_parent(col1) on delete cascade on update cascade
);

alter table convslot_test_child add column col2 text not null default 'tutu';

insert into convslot_test_parent(col1) values ('1');

insert into convslot_test_child(col1) values ('1');

insert into convslot_test_parent(col1) values ('3');

insert into convslot_test_child(col1) values ('3');

create function convslot_trig1()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, old_table = %',
          TG_NAME,
          (select string_agg(old_table::text, ', ' order by col1) from old_table);
return null;
end; $$;

create function convslot_trig2()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, new table = %',
          TG_NAME,
          (select string_agg(new_table::text, ', ' order by col1) from new_table);
return null;
end; $$;

create trigger but_trigger after update on convslot_test_child
referencing new table as new_table
for each statement execute function convslot_trig2();

update convslot_test_parent set col1 = col1 || '1';

create function convslot_trig3()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, old_table = %, new table = %',
          TG_NAME,
          (select string_agg(old_table::text, ', ' order by col1) from old_table),
          (select string_agg(new_table::text, ', ' order by col1) from new_table);
return null;
end; $$;

create trigger but_trigger2 after update on convslot_test_child
referencing old table as old_table new table as new_table
for each statement execute function convslot_trig3();

update convslot_test_parent set col1 = col1 || '1';

create trigger bdt_trigger after delete on convslot_test_child
referencing old table as old_table
for each statement execute function convslot_trig1();

delete from convslot_test_parent;

drop table convslot_test_child, convslot_test_parent;

drop function convslot_trig1();

drop function convslot_trig2();

drop function convslot_trig3();

create table convslot_test_parent (id int primary key, val int)
partition by range (id);

create table convslot_test_part (val int, id int not null);

alter table convslot_test_parent
  attach partition convslot_test_part for values from (1) to (1000);

create function convslot_trig4() returns trigger as
$$begin raise exception 'BOOM!'; end$$ language plpgsql;

create trigger convslot_test_parent_update
    after update on convslot_test_parent
    referencing old table as old_rows new table as new_rows
    for each statement execute procedure convslot_trig4();

insert into convslot_test_parent (id, val) values (1, 2);

begin;

savepoint svp;

update convslot_test_parent set val = 3;

rollback to savepoint svp;

rollback;

drop table convslot_test_parent;

drop function convslot_trig4();

create table grandparent (id int, primary key (id)) partition by range (id);

create table middle partition of grandparent for values from (1) to (10)
partition by range (id);

create table chi partition of middle for values from (1) to (5);

create table cho partition of middle for values from (6) to (10);

create function f () returns trigger as
$$ begin return new; end; $$
language plpgsql;

create trigger a after insert on grandparent
for each row execute procedure f();

alter trigger a on grandparent rename to b;

select tgrelid::regclass, tgname,
(select tgname from pg_trigger tr where tr.oid = pg_trigger.tgparentid) parent_tgname
from pg_trigger where tgrelid in (select relid from pg_partition_tree('grandparent'))
order by tgname, tgrelid::regclass::text COLLATE "C";

alter trigger b on middle rename to c;

create trigger c after insert on middle
for each row execute procedure f();

alter trigger b on grandparent rename to c;

create trigger p after insert on grandparent for each statement execute function f();

create trigger p after insert on middle for each statement execute function f();

alter trigger p on grandparent rename to q;

select tgrelid::regclass, tgname,
(select tgname from pg_trigger tr where tr.oid = pg_trigger.tgparentid) parent_tgname
from pg_trigger where tgrelid in (select relid from pg_partition_tree('grandparent'))
order by tgname, tgrelid::regclass::text COLLATE "C";

drop table grandparent;

create table parent (a int);

create table child () inherits (parent);

create trigger parenttrig after insert on parent
for each row execute procedure f();

create trigger parenttrig after insert on child
for each row execute procedure f();

alter trigger parenttrig on parent rename to anothertrig;

drop table parent, child;

drop function f();

create role regress_caller;

create role regress_fn_owner;

create function whoami() returns trigger language plpgsql
as $$
begin
  raise notice 'I am %', current_user;
  return null;
end;
$$;

alter function whoami() owner to regress_fn_owner;

create table defer_trig (id integer);

grant insert on defer_trig to public;

create constraint trigger whoami after insert on defer_trig
  deferrable initially deferred
  for each row
  execute function whoami();

begin;

set role regress_caller;

insert into defer_trig values (1);

reset role;

set role regress_fn_owner;

insert into defer_trig values (2);

reset role;

commit;

alter function whoami() security definer;

begin;

set role regress_caller;

insert into defer_trig values (3);

reset role;

commit;

alter function whoami() security invoker;

create or replace function whoami() returns trigger language plpgsql
as $$
begin
  raise notice 'I am %', current_user;
  perform 1 / 0;
  return null;
end;
$$;

begin;

set role regress_caller;

insert into defer_trig values (4);

reset role;

commit;

select current_user = session_user;

drop table defer_trig;

drop function whoami();

drop role regress_fn_owner;

drop role regress_caller;
