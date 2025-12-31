CREATE USER regress_merge_privs;

CREATE USER regress_merge_no_privs;

CREATE USER regress_merge_none;

DROP TABLE IF EXISTS target;

DROP TABLE IF EXISTS source;

CREATE TABLE target (tid integer, balance integer)
  WITH (autovacuum_enabled=off);

CREATE TABLE source (sid integer, delta integer) -- no index
  WITH (autovacuum_enabled=off);

INSERT INTO target VALUES (1, 10);

INSERT INTO target VALUES (2, 20);

INSERT INTO target VALUES (3, 30);

SELECT t.ctid is not null as matched, t.*, s.* FROM source s FULL OUTER JOIN target t ON s.sid = t.tid ORDER BY t.tid, s.sid;

ALTER TABLE target OWNER TO regress_merge_privs;

ALTER TABLE source OWNER TO regress_merge_privs;

CREATE TABLE target2 (tid integer, balance integer)
  WITH (autovacuum_enabled=off);

CREATE TABLE source2 (sid integer, delta integer)
  WITH (autovacuum_enabled=off);

ALTER TABLE target2 OWNER TO regress_merge_no_privs;

ALTER TABLE source2 OWNER TO regress_merge_no_privs;

GRANT INSERT ON target TO regress_merge_no_privs;

SET SESSION AUTHORIZATION regress_merge_privs;

INSERT INTO target DEFAULT VALUES;

UPDATE target SET balance = 0;

MERGE INTO target
USING target
ON tid = tid
WHEN MATCHED THEN DO NOTHING;

WITH foo AS (
  MERGE INTO target USING source ON (true)
  WHEN MATCHED THEN DELETE
) SELECT * FROM foo;

COPY (
  MERGE INTO target USING source ON (true)
  WHEN MATCHED THEN DELETE
) TO stdout;

CREATE MATERIALIZED VIEW mv AS SELECT * FROM target;

DROP MATERIALIZED VIEW mv;

SET SESSION AUTHORIZATION regress_merge_none;

MERGE INTO target
USING (SELECT 1)
ON true
WHEN MATCHED THEN
	DO NOTHING;

SET SESSION AUTHORIZATION regress_merge_privs;

GRANT INSERT ON target TO regress_merge_no_privs;

SET SESSION AUTHORIZATION regress_merge_no_privs;

GRANT UPDATE ON target2 TO regress_merge_privs;

SET SESSION AUTHORIZATION regress_merge_privs;

BEGIN;

ROLLBACK;

INSERT INTO source VALUES (4, 40);

SELECT * FROM source ORDER BY sid;

SELECT * FROM target ORDER BY tid;

MERGE INTO target t
USING source AS s
ON t.tid = s.sid
WHEN NOT MATCHED THEN
	DO NOTHING;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

INSERT INTO target SELECT generate_series(1000,2500), 0;

ALTER TABLE target ADD PRIMARY KEY (tid);

ANALYZE target;

DELETE FROM target WHERE tid > 100;

ANALYZE target;

INSERT INTO source VALUES (2, 5);

INSERT INTO source VALUES (3, 20);

SELECT * FROM source ORDER BY sid;

SELECT * FROM target ORDER BY tid;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

MERGE INTO target t
USING source AS s
ON t.tid = s.sid
WHEN MATCHED THEN
	DO NOTHING;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

INSERT INTO source VALUES (2, 5);

SELECT * FROM source ORDER BY sid;

SELECT * FROM target ORDER BY tid;

BEGIN;

ROLLBACK;

BEGIN;

ROLLBACK;

DELETE FROM source WHERE sid = 2;

INSERT INTO source VALUES (2, 5);

SELECT * FROM source ORDER BY sid;

SELECT * FROM target ORDER BY tid;

INSERT INTO source VALUES (4, 40);

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

DELETE FROM source WHERE sid = 4;

INSERT INTO source VALUES (4, 40);

SELECT * FROM source ORDER BY sid;

SELECT * FROM target ORDER BY tid;

alter table target drop CONSTRAINT target_pkey;

alter table target alter column tid drop not null;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

INSERT INTO source VALUES (5, 50);

INSERT INTO source VALUES (5, 50);

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

DELETE FROM source WHERE sid = 5;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

ROLLBACK;

CREATE TABLE wq_target (tid integer not null, balance integer DEFAULT -1)
  WITH (autovacuum_enabled=off);

CREATE TABLE wq_source (balance integer, sid integer)
  WITH (autovacuum_enabled=off);

INSERT INTO wq_source (sid, balance) VALUES (1, 100);

BEGIN;

SELECT * FROM wq_target;

ROLLBACK;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

BEGIN;

SELECT * FROM wq_target;

ROLLBACK;

BEGIN;

SELECT * FROM wq_target;

ROLLBACK;

SELECT * FROM wq_target;

SELECT * FROM wq_source;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

SELECT * FROM wq_target;

BEGIN;

SELECT * FROM wq_target;

ROLLBACK;

SELECT * FROM wq_target;

DROP TABLE wq_target, wq_source;

create or replace function merge_trigfunc () returns trigger
language plpgsql as
$$
DECLARE
	line text;
BEGIN
	SELECT INTO line format('%s %s %s trigger%s',
		TG_WHEN, TG_OP, TG_LEVEL, CASE
		WHEN TG_OP = 'INSERT' AND TG_LEVEL = 'ROW'
			THEN format(' row: %s', NEW)
		WHEN TG_OP = 'UPDATE' AND TG_LEVEL = 'ROW'
			THEN format(' row: %s -> %s', OLD, NEW)
		WHEN TG_OP = 'DELETE' AND TG_LEVEL = 'ROW'
			THEN format(' row: %s', OLD)
		END);

	RAISE NOTICE '%', line;
	IF (TG_WHEN = 'BEFORE' AND TG_LEVEL = 'ROW') THEN
		IF (TG_OP = 'DELETE') THEN
			RETURN OLD;
		ELSE
			RETURN NEW;
		END IF;
	ELSE
		RETURN NULL;
	END IF;
END;
$$;

CREATE TRIGGER merge_bsi BEFORE INSERT ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_bsu BEFORE UPDATE ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_bsd BEFORE DELETE ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_asi AFTER INSERT ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_asu AFTER UPDATE ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_asd AFTER DELETE ON target FOR EACH STATEMENT EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_bri BEFORE INSERT ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_bru BEFORE UPDATE ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_brd BEFORE DELETE ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_ari AFTER INSERT ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_aru AFTER UPDATE ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

CREATE TRIGGER merge_ard AFTER DELETE ON target FOR EACH ROW EXECUTE PROCEDURE merge_trigfunc ();

BEGIN;

UPDATE target SET balance = 0 WHERE tid = 3;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

DELETE FROM SOURCE WHERE sid = 2;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

create or replace function skip_merge_op() returns trigger
language plpgsql as
$$
BEGIN
	RETURN NULL;
END;
$$;

SELECT * FROM target full outer join source on (sid = tid);

create trigger merge_skip BEFORE INSERT OR UPDATE or DELETE
  ON target FOR EACH ROW EXECUTE FUNCTION skip_merge_op();

DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO target t
USING source AS s
ON t.tid = s.sid
WHEN MATCHED AND s.sid = 3 THEN UPDATE SET balance = t.balance + s.delta
WHEN MATCHED THEN DELETE
WHEN NOT MATCHED THEN INSERT VALUES (sid, delta);
IF FOUND THEN
  RAISE NOTICE 'Found';
ELSE
  RAISE NOTICE 'Not found';
END IF;
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;

SELECT * FROM target FULL OUTER JOIN source ON (sid = tid);

DROP TRIGGER merge_skip ON target;

DROP FUNCTION skip_merge_op();

BEGIN;

DO LANGUAGE plpgsql $$
BEGIN
MERGE INTO target t
USING source AS s
ON t.tid = s.sid
WHEN MATCHED AND t.balance > s.delta THEN
	UPDATE SET balance = t.balance - s.delta;
END;
$$;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

CREATE FUNCTION merge_func (p_id integer, p_bal integer)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
 result integer;
BEGIN
MERGE INTO target t
USING (SELECT p_id AS sid) AS s
ON t.tid = s.sid
WHEN MATCHED THEN
	UPDATE SET balance = t.balance - p_bal;
IF FOUND THEN
	GET DIAGNOSTICS result := ROW_COUNT;
END IF;
RETURN result;
END;
$$;

SELECT merge_func(3, 4);

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

execute foom;

SELECT * FROM target ORDER BY tid;

ROLLBACK;

BEGIN;

execute foom2 (1, 1);

SELECT * FROM target ORDER BY tid;

ROLLBACK;

CREATE TABLE sq_target (tid integer NOT NULL, balance integer)
  WITH (autovacuum_enabled=off);

CREATE TABLE sq_source (delta integer, sid integer, balance integer DEFAULT 0)
  WITH (autovacuum_enabled=off);

INSERT INTO sq_target(tid, balance) VALUES (1,100), (2,200), (3,300);

INSERT INTO sq_source(sid, delta) VALUES (1,10), (2,20), (4,40);

BEGIN;

SELECT * FROM sq_target;

ROLLBACK;

CREATE VIEW v AS SELECT * FROM sq_source WHERE sid < 2;

BEGIN;

SELECT * FROM sq_target;

ROLLBACK;

BEGIN;

ROLLBACK;

BEGIN;

INSERT INTO sq_source (sid, balance, delta) VALUES (-1, -1, -10);

SELECT * FROM sq_target;

ROLLBACK;

BEGIN;

INSERT INTO sq_source (sid, balance, delta) VALUES (-1, -1, -10);

WITH targq AS (
	SELECT * FROM v
)
MERGE INTO sq_target t
USING v
ON tid = sid
WHEN MATCHED AND tid >= 2 THEN
    UPDATE SET balance = t.balance + delta
WHEN NOT MATCHED THEN
	INSERT (balance, tid) VALUES (balance + delta, sid)
WHEN MATCHED AND tid < 2 THEN
	DELETE;

ROLLBACK;

SELECT * FROM sq_source ORDER BY sid;

SELECT * FROM sq_target ORDER BY tid;

BEGIN;

CREATE TABLE merge_actions(action text, abbrev text);

INSERT INTO merge_actions VALUES ('INSERT', 'ins'), ('UPDATE', 'upd'), ('DELETE', 'del');

ROLLBACK;

SELECT merge_action() FROM sq_target;

UPDATE sq_target SET balance = balance + 1 RETURNING merge_action();

CREATE TABLE sq_target_merge_log (tid integer NOT NULL, last_change text);

INSERT INTO sq_target_merge_log VALUES (1, 'Original value');

BEGIN;

WITH m AS (
    MERGE INTO sq_target t
    USING sq_source s
    ON tid = sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (balance + delta, sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action() AS action, old AS old_data, new AS new_data, t.*,
              CASE merge_action()
                  WHEN 'INSERT' THEN 'Inserted '||t
                  WHEN 'UPDATE' THEN 'Added '||delta||' to balance'
                  WHEN 'DELETE' THEN 'Removed '||t
              END AS description
), m2 AS (
    MERGE INTO sq_target_merge_log l
    USING m
    ON l.tid = m.tid
    WHEN MATCHED THEN
        UPDATE SET last_change = description
    WHEN NOT MATCHED THEN
        INSERT VALUES (m.tid, description)
    RETURNING m.*, merge_action() AS log_action, old AS old_log, new AS new_log, l.*
)
SELECT * FROM m2;

SELECT * FROM sq_target_merge_log ORDER BY tid;

ROLLBACK;

BEGIN;

COPY (
    MERGE INTO sq_target t
    USING sq_source s
    ON tid = sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (balance + delta, sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), old.*, new.*
) TO stdout;

ROLLBACK;

BEGIN;

CREATE FUNCTION merge_into_sq_target(sid int, balance int, delta int,
                                     OUT action text, OUT tid int, OUT new_balance int)
LANGUAGE sql AS
$$
    MERGE INTO sq_target t
    USING (VALUES ($1, $2, $3)) AS v(sid, balance, delta)
    ON tid = v.sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + v.delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (v.balance + v.delta, v.sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), t.*;
$$;

SELECT m.*
FROM (VALUES (1, 0, 0), (3, 0, 20), (4, 100, 10)) AS v(sid, balance, delta),
LATERAL (SELECT action, tid, new_balance FROM merge_into_sq_target(sid, balance, delta)) m;

ROLLBACK;

BEGIN;

CREATE FUNCTION merge_sq_source_into_sq_target()
RETURNS TABLE (action text, tid int, balance int)
LANGUAGE sql AS
$$
    MERGE INTO sq_target t
    USING sq_source s
    ON tid = sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (balance + delta, sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), t.*;
$$;

SELECT * FROM merge_sq_source_into_sq_target();

ROLLBACK;

BEGIN;

CREATE FUNCTION merge_into_sq_target(sid int, balance int, delta int,
                                     OUT r_action text, OUT r_tid int, OUT r_balance int)
LANGUAGE plpgsql AS
$$
BEGIN
    MERGE INTO sq_target t
    USING (VALUES ($1, $2, $3)) AS v(sid, balance, delta)
    ON tid = v.sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + v.delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (v.balance + v.delta, v.sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), t.* INTO r_action, r_tid, r_balance;
END;
$$;

SELECT m.*
FROM (VALUES (1, 0, 0), (3, 0, 20), (4, 100, 10)) AS v(sid, balance, delta),
LATERAL (SELECT r_action, r_tid, r_balance FROM merge_into_sq_target(sid, balance, delta)) m;

ROLLBACK;

CREATE TABLE ex_mtarget (a int, b int)
  WITH (autovacuum_enabled=off);

CREATE TABLE ex_msource (a int, b int)
  WITH (autovacuum_enabled=off);

INSERT INTO ex_mtarget SELECT i, i*10 FROM generate_series(1,100,2) i;

INSERT INTO ex_msource SELECT i, i*10 FROM generate_series(1,100,1) i;

CREATE FUNCTION explain_merge(query text) RETURNS SETOF text
LANGUAGE plpgsql AS
$$
DECLARE ln text;
BEGIN
    FOR ln IN
        EXECUTE 'explain (analyze, timing off, summary off, costs off, buffers off) ' ||
		  query
    LOOP
        ln := regexp_replace(ln, '(Memory( Usage)?|Buckets|Batches): \S*',  '\1: xxx', 'g');
        RETURN NEXT ln;
    END LOOP;
END;
$$;

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN MATCHED THEN
	UPDATE SET b = t.b + 1');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN MATCHED AND t.a < 10 THEN
	UPDATE SET b = t.b + 1');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN MATCHED AND t.a < 10 THEN
	UPDATE SET b = t.b + 1
WHEN MATCHED AND t.a >= 10 AND t.a <= 20 THEN
	DELETE');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN NOT MATCHED AND s.a < 10 THEN
	INSERT VALUES (a, b)');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN MATCHED AND t.a < 10 THEN
	UPDATE SET b = t.b + 1
WHEN MATCHED AND t.a >= 30 AND t.a <= 40 THEN
	DELETE
WHEN NOT MATCHED AND s.a < 20 THEN
	INSERT VALUES (a, b)');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN NOT MATCHED BY SOURCE and t.a < 10 THEN
	DELETE');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN NOT MATCHED BY SOURCE AND t.a < 10 THEN
	DELETE
WHEN NOT MATCHED BY TARGET AND s.a < 20 THEN
	INSERT VALUES (a, b)');

SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a AND t.a < -1000
WHEN MATCHED AND t.a < 10 THEN
	DO NOTHING');

DROP TABLE ex_msource, ex_mtarget;

DROP FUNCTION explain_merge(text);

CREATE TABLE src (a int, b int, c int, d int);

CREATE TABLE tgt (a int, b int, c int, d int);

CREATE TABLE ref (ab int, cd int);

DROP TABLE src, tgt, ref;

BEGIN;

SELECT * FROM sq_target WHERE tid = 1;

ROLLBACK;

BEGIN;

SELECT * FROM sq_target WHERE tid = 1;

ROLLBACK;

BEGIN;

SELECT * FROM sq_target WHERE tid = 1;

ROLLBACK;

DROP TABLE sq_target, sq_target_merge_log, sq_source CASCADE;

CREATE TABLE pa_target (tid integer, balance float, val text)
	PARTITION BY LIST (tid);

CREATE TABLE part1 PARTITION OF pa_target FOR VALUES IN (1,4)
  WITH (autovacuum_enabled=off);

CREATE TABLE part2 PARTITION OF pa_target FOR VALUES IN (2,5,6)
  WITH (autovacuum_enabled=off);

CREATE TABLE part3 PARTITION OF pa_target FOR VALUES IN (3,8,9)
  WITH (autovacuum_enabled=off);

CREATE TABLE part4 PARTITION OF pa_target DEFAULT
  WITH (autovacuum_enabled=off);

CREATE TABLE pa_source (sid integer, delta float);

INSERT INTO pa_source SELECT id, id * 10  FROM generate_series(1,14) AS id;

INSERT INTO pa_target SELECT id, id * 100, 'initial' FROM generate_series(1,15,2) AS id;

BEGIN;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

CREATE FUNCTION merge_func() RETURNS integer LANGUAGE plpgsql AS $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET tid = tid + 1, balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET tid = 1, val = val || ' not matched by source';
IF FOUND THEN
  GET DIAGNOSTICS result := ROW_COUNT;
END IF;
RETURN result;
END;
$$;

SELECT merge_func();

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

SELECT * FROM pa_target ORDER BY tid;

ROLLBACK;

BEGIN;

TRUNCATE pa_target;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

DROP TABLE pa_target CASCADE;

CREATE TABLE pa_target (tid integer, balance float, val text)
	PARTITION BY LIST (tid);

CREATE TABLE part1 (tid integer, balance float, val text)
  WITH (autovacuum_enabled=off);

CREATE TABLE part2 (balance float, tid integer, val text)
  WITH (autovacuum_enabled=off);

CREATE TABLE part3 (tid integer, balance float, val text)
  WITH (autovacuum_enabled=off);

CREATE TABLE part4 (extraid text, tid integer, balance float, val text)
  WITH (autovacuum_enabled=off);

ALTER TABLE part4 DROP COLUMN extraid;

ALTER TABLE pa_target ATTACH PARTITION part1 FOR VALUES IN (1,4);

ALTER TABLE pa_target ATTACH PARTITION part2 FOR VALUES IN (2,5,6);

ALTER TABLE pa_target ATTACH PARTITION part3 FOR VALUES IN (3,8,9);

ALTER TABLE pa_target ATTACH PARTITION part4 DEFAULT;

INSERT INTO pa_target SELECT id, id * 100, 'initial' FROM generate_series(1,15,2) AS id;

BEGIN;

DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET val = val || ' not matched by source';
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET tid = tid + 1, balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET tid = 1, val = val || ' not matched by source';
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

CREATE FUNCTION trig_fn() RETURNS trigger LANGUAGE plpgsql AS
  $$ BEGIN RETURN NULL; END; $$;

CREATE TRIGGER del_trig BEFORE DELETE ON pa_target
  FOR EACH ROW EXECUTE PROCEDURE trig_fn();

DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET tid = tid + 1, balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET val = val || ' not matched by source';
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

CREATE FUNCTION trig_fn() RETURNS trigger LANGUAGE plpgsql AS
  $$ BEGIN RETURN NULL; END; $$;

CREATE TRIGGER ins_trig BEFORE INSERT ON pa_target
  FOR EACH ROW EXECUTE PROCEDURE trig_fn();

DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET tid = tid + 1, balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET val = val || ' not matched by source';
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;

SELECT * FROM pa_target ORDER BY tid, val;

ROLLBACK;

BEGIN;

ALTER TABLE pa_target ENABLE ROW LEVEL SECURITY;

ALTER TABLE pa_target FORCE ROW LEVEL SECURITY;

CREATE POLICY pa_target_pol ON pa_target USING (tid != 0);

ROLLBACK;

DROP TABLE pa_source;

DROP TABLE pa_target CASCADE;

CREATE TABLE pa_target (logts timestamp, tid integer, balance float, val text)
	PARTITION BY RANGE (logts);

CREATE TABLE part_m01 PARTITION OF pa_target
	FOR VALUES FROM ('2017-01-01') TO ('2017-02-01')
	PARTITION BY LIST (tid);

CREATE TABLE part_m01_odd PARTITION OF part_m01
	FOR VALUES IN (1,3,5,7,9) WITH (autovacuum_enabled=off);

CREATE TABLE part_m01_even PARTITION OF part_m01
	FOR VALUES IN (2,4,6,8) WITH (autovacuum_enabled=off);

CREATE TABLE part_m02 PARTITION OF pa_target
	FOR VALUES FROM ('2017-02-01') TO ('2017-03-01')
	PARTITION BY LIST (tid);

CREATE TABLE part_m02_odd PARTITION OF part_m02
	FOR VALUES IN (1,3,5,7,9) WITH (autovacuum_enabled=off);

CREATE TABLE part_m02_even PARTITION OF part_m02
	FOR VALUES IN (2,4,6,8) WITH (autovacuum_enabled=off);

CREATE TABLE pa_source (sid integer, delta float)
  WITH (autovacuum_enabled=off);

INSERT INTO pa_source SELECT id, id * 10  FROM generate_series(1,14) AS id;

INSERT INTO pa_target SELECT '2017-01-31', id, id * 100, 'initial' FROM generate_series(1,9,3) AS id;

INSERT INTO pa_target SELECT '2017-02-28', id, id * 100, 'initial' FROM generate_series(2,9,3) AS id;

BEGIN;

SELECT * FROM pa_target ORDER BY tid;

ROLLBACK;

DROP TABLE pa_source;

DROP TABLE pa_target CASCADE;

CREATE TABLE pa_target (tid integer PRIMARY KEY) PARTITION BY LIST (tid);

CREATE TABLE pa_targetp PARTITION OF pa_target DEFAULT;

CREATE TABLE pa_source (sid integer);

INSERT INTO pa_source VALUES (1), (2);

TABLE pa_target;

DROP TABLE pa_targetp;

DROP TABLE pa_source;

DROP TABLE pa_target CASCADE;

CREATE TABLE cj_target (tid integer, balance float, val text)
  WITH (autovacuum_enabled=off);

CREATE TABLE cj_source1 (sid1 integer, scat integer, delta integer)
  WITH (autovacuum_enabled=off);

CREATE TABLE cj_source2 (sid2 integer, sval text)
  WITH (autovacuum_enabled=off);

INSERT INTO cj_source1 VALUES (1, 10, 100);

INSERT INTO cj_source1 VALUES (1, 20, 200);

INSERT INTO cj_source1 VALUES (2, 20, 300);

INSERT INTO cj_source1 VALUES (3, 10, 400);

INSERT INTO cj_source2 VALUES (1, 'initial source2');

INSERT INTO cj_source2 VALUES (2, 'initial source2');

INSERT INTO cj_source2 VALUES (3, 'initial source2');

SELECT * FROM cj_target;

SELECT * FROM cj_target;

ALTER TABLE cj_source1 RENAME COLUMN sid1 TO sid;

ALTER TABLE cj_source2 RENAME COLUMN sid2 TO sid;

TRUNCATE cj_target;

DROP TABLE cj_source2, cj_source1, cj_target;

CREATE TABLE fs_target (a int, b int, c text)
  WITH (autovacuum_enabled=off);

SELECT count(*) FROM fs_target;

DROP TABLE fs_target;

CREATE TABLE measurement (
    city_id         int not null,
    logdate         date not null,
    peaktemp        int,
    unitsales       int
) WITH (autovacuum_enabled=off);

CREATE TABLE measurement_y2006m02 (
    CHECK ( logdate >= DATE '2006-02-01' AND logdate < DATE '2006-03-01' )
) INHERITS (measurement) WITH (autovacuum_enabled=off);

CREATE TABLE measurement_y2006m03 (
    CHECK ( logdate >= DATE '2006-03-01' AND logdate < DATE '2006-04-01' )
) INHERITS (measurement) WITH (autovacuum_enabled=off);

CREATE TABLE measurement_y2007m01 (
    filler          text,
    peaktemp        int,
    logdate         date not null,
    city_id         int not null,
    unitsales       int
    CHECK ( logdate >= DATE '2007-01-01' AND logdate < DATE '2007-02-01')
) WITH (autovacuum_enabled=off);

ALTER TABLE measurement_y2007m01 DROP COLUMN filler;

ALTER TABLE measurement_y2007m01 INHERIT measurement;

INSERT INTO measurement VALUES (0, '2005-07-21', 5, 15);

CREATE OR REPLACE FUNCTION measurement_insert_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF ( NEW.logdate >= DATE '2006-02-01' AND
         NEW.logdate < DATE '2006-03-01' ) THEN
        INSERT INTO measurement_y2006m02 VALUES (NEW.*);
    ELSIF ( NEW.logdate >= DATE '2006-03-01' AND
            NEW.logdate < DATE '2006-04-01' ) THEN
        INSERT INTO measurement_y2006m03 VALUES (NEW.*);
    ELSIF ( NEW.logdate >= DATE '2007-01-01' AND
            NEW.logdate < DATE '2007-02-01' ) THEN
        INSERT INTO measurement_y2007m01 (city_id, logdate, peaktemp, unitsales)
            VALUES (NEW.*);
    ELSE
        RAISE EXCEPTION 'Date out of range.  Fix the measurement_insert_trigger() function!';
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql ;

CREATE TRIGGER insert_measurement_trigger
    BEFORE INSERT ON measurement
    FOR EACH ROW EXECUTE PROCEDURE measurement_insert_trigger();

INSERT INTO measurement VALUES (1, '2006-02-10', 35, 10);

INSERT INTO measurement VALUES (1, '2006-02-16', 45, 20);

INSERT INTO measurement VALUES (1, '2006-03-17', 25, 10);

INSERT INTO measurement VALUES (1, '2006-03-27', 15, 40);

INSERT INTO measurement VALUES (1, '2007-01-15', 10, 10);

INSERT INTO measurement VALUES (1, '2007-01-17', 10, 10);

SELECT tableoid::regclass, * FROM measurement ORDER BY city_id, logdate;

CREATE TABLE new_measurement (LIKE measurement) WITH (autovacuum_enabled=off);

INSERT INTO new_measurement VALUES (0, '2005-07-21', 25, 20);

INSERT INTO new_measurement VALUES (1, '2006-03-01', 20, 10);

INSERT INTO new_measurement VALUES (1, '2006-02-16', 50, 10);

INSERT INTO new_measurement VALUES (2, '2006-02-10', 20, 20);

INSERT INTO new_measurement VALUES (1, '2006-03-27', NULL, NULL);

INSERT INTO new_measurement VALUES (1, '2007-01-17', NULL, NULL);

INSERT INTO new_measurement VALUES (1, '2007-01-15', 5, NULL);

INSERT INTO new_measurement VALUES (1, '2007-01-16', 10, 10);

BEGIN;

SELECT tableoid::regclass, * FROM measurement ORDER BY city_id, logdate, peaktemp;

ROLLBACK;

SELECT tableoid::regclass, * FROM measurement ORDER BY city_id, logdate;

BEGIN;

SELECT * FROM new_measurement ORDER BY city_id, logdate;

ROLLBACK;

SELECT * FROM new_measurement ORDER BY city_id, logdate;

DROP TRIGGER insert_measurement_trigger ON measurement;

ALTER TABLE measurement ADD CONSTRAINT mcheck CHECK (city_id = 0) NO INHERIT;

BEGIN;

SELECT * FROM ONLY measurement ORDER BY city_id, logdate;

ROLLBACK;

ALTER TABLE measurement ENABLE ROW LEVEL SECURITY;

ALTER TABLE measurement FORCE ROW LEVEL SECURITY;

CREATE POLICY measurement_p ON measurement USING (peaktemp IS NOT NULL);

SELECT * FROM ONLY measurement ORDER BY city_id, logdate;

DROP TABLE measurement, new_measurement CASCADE;

DROP FUNCTION measurement_insert_trigger();

CREATE TABLE src (a int, b text);

INSERT INTO src VALUES (1, 'src row');

CREATE TABLE tgt (a int, b text);

INSERT INTO tgt VALUES (NULL, 'tgt row');

SELECT * FROM tgt;

DROP TABLE src, tgt;

CREATE TABLE bug18634t (a int, b int, c text);

INSERT INTO bug18634t VALUES(1, 10, 'tgt1'), (2, 20, 'tgt2');

CREATE VIEW bug18634v AS
  SELECT * FROM bug18634t WHERE EXISTS (SELECT 1 FROM bug18634t);

CREATE TABLE bug18634s (a int, b int, c text);

INSERT INTO bug18634s VALUES (1, 2, 'src1');

SELECT * FROM bug18634t;

DROP TABLE bug18634t CASCADE;

DROP TABLE bug18634s;

RESET SESSION AUTHORIZATION;

CREATE VIEW classv AS SELECT * FROM pg_class;

DROP TABLE target, target2;

DROP TABLE source, source2;

DROP FUNCTION merge_trigfunc();

DROP USER regress_merge_privs;

DROP USER regress_merge_no_privs;

DROP USER regress_merge_none;
