SET extra_float_digits = 0;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW ro_view1 AS SELECT DISTINCT a, b FROM base_tbl;

CREATE VIEW ro_view2 AS SELECT a, b FROM base_tbl GROUP BY a, b;

CREATE VIEW ro_view3 AS SELECT 1 FROM base_tbl HAVING max(a) > 0;

CREATE VIEW ro_view4 AS SELECT count(*) FROM base_tbl;

CREATE VIEW ro_view5 AS SELECT a, rank() OVER() FROM base_tbl;

CREATE VIEW ro_view6 AS SELECT a, b FROM base_tbl UNION SELECT -a, b FROM base_tbl;

SELECT * FROM t;

CREATE VIEW ro_view8 AS SELECT a, b FROM base_tbl ORDER BY a OFFSET 1;

CREATE VIEW ro_view9 AS SELECT a, b FROM base_tbl ORDER BY a LIMIT 1;

CREATE VIEW ro_view10 AS SELECT 1 AS a;

CREATE VIEW ro_view11 AS SELECT b1.a, b2.b FROM base_tbl b1, base_tbl b2;

CREATE VIEW ro_view12 AS SELECT * FROM generate_series(1, 10) AS g(a);

CREATE VIEW ro_view13 AS SELECT a, b FROM (SELECT * FROM base_tbl) AS t;

CREATE VIEW rw_view14 AS SELECT ctid, a, b FROM base_tbl;

CREATE VIEW rw_view15 AS SELECT a, upper(b) FROM base_tbl;

CREATE VIEW rw_view16 AS SELECT a, b, a AS aa FROM base_tbl;

CREATE VIEW ro_view17 AS SELECT * FROM ro_view1;

CREATE VIEW ro_view18 AS SELECT * FROM (VALUES(1)) AS tmp(a);

CREATE SEQUENCE uv_seq;

CREATE VIEW ro_view19 AS SELECT * FROM uv_seq;

CREATE VIEW ro_view20 AS SELECT a, b, generate_series(1, a) g FROM base_tbl;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name, ordinal_position;

DELETE FROM ro_view1;

DELETE FROM ro_view2;

DELETE FROM ro_view3;

DELETE FROM ro_view4;

DELETE FROM ro_view5;

DELETE FROM ro_view6;

UPDATE ro_view7 SET a=a+1;

UPDATE ro_view8 SET a=a+1;

UPDATE ro_view9 SET a=a+1;

UPDATE ro_view10 SET a=a+1;

UPDATE ro_view11 SET a=a+1;

UPDATE ro_view12 SET a=a+1;

INSERT INTO ro_view13 VALUES (3, 'Row 3');

MERGE INTO ro_view13 AS t USING (VALUES (2, 'Row 2')) AS v(a,b) ON t.a = v.a
  WHEN MATCHED THEN DO NOTHING
  WHEN NOT MATCHED THEN DO NOTHING;

MERGE INTO ro_view13 AS t USING (VALUES (3, 'Row 3')) AS v(a,b) ON t.a = v.a
  WHEN MATCHED THEN DO NOTHING
  WHEN NOT MATCHED THEN DO NOTHING;

INSERT INTO rw_view14 VALUES (null, 3, 'Row 3');

INSERT INTO rw_view14 (a, b) VALUES (3, 'Row 3');

UPDATE rw_view14 SET ctid=null WHERE a=3;

UPDATE rw_view14 SET b='ROW 3' WHERE a=3;

SELECT * FROM base_tbl;

DELETE FROM rw_view14 WHERE a=3;

SELECT * FROM base_tbl ORDER BY a;

SELECT * FROM base_tbl ORDER BY a;

INSERT INTO rw_view15 VALUES (3, 'ROW 3');

INSERT INTO rw_view15 (a) VALUES (3);

INSERT INTO rw_view15 (a) VALUES (3) ON CONFLICT DO NOTHING;

SELECT * FROM rw_view15;

INSERT INTO rw_view15 (a) VALUES (3) ON CONFLICT (a) DO NOTHING;

SELECT * FROM rw_view15;

INSERT INTO rw_view15 (a) VALUES (3) ON CONFLICT (a) DO UPDATE set a = excluded.a;

SELECT * FROM rw_view15;

INSERT INTO rw_view15 (a) VALUES (3) ON CONFLICT (a) DO UPDATE set upper = 'blarg';

SELECT * FROM rw_view15;

SELECT * FROM rw_view15;

ALTER VIEW rw_view15 ALTER COLUMN upper SET DEFAULT 'NOT SET';

INSERT INTO rw_view15 (a) VALUES (4);

UPDATE rw_view15 SET upper='ROW 3' WHERE a=3;

UPDATE rw_view15 SET upper=DEFAULT WHERE a=3;

UPDATE rw_view15 SET a=4 WHERE a=3;

SELECT * FROM base_tbl;

DELETE FROM rw_view15 WHERE a=4;

INSERT INTO rw_view16 VALUES (3, 'Row 3', 3);

INSERT INTO rw_view16 (a, b) VALUES (3, 'Row 3');

UPDATE rw_view16 SET a=3, aa=-3 WHERE a=3;

UPDATE rw_view16 SET aa=-3 WHERE a=3;

SELECT * FROM base_tbl;

DELETE FROM rw_view16 WHERE a=-3;

INSERT INTO ro_view17 VALUES (3, 'ROW 3');

DELETE FROM ro_view18;

MERGE INTO ro_view18 AS t USING (VALUES (1, 'Row 1')) AS v(a,b) ON t.a = v.a
  WHEN MATCHED THEN DO NOTHING;

UPDATE ro_view19 SET last_value=1000;

UPDATE ro_view20 SET b=upper(b);

CREATE RULE rw_view16_ins_rule AS ON INSERT TO rw_view16
  WHERE NEW.a > 0 DO INSTEAD INSERT INTO base_tbl VALUES (NEW.a, NEW.b);

CREATE RULE rw_view16_upd_rule AS ON UPDATE TO rw_view16
  WHERE OLD.a > 0 DO INSTEAD UPDATE base_tbl SET b=NEW.b WHERE a=OLD.a;

CREATE RULE rw_view16_del_rule AS ON DELETE TO rw_view16
  WHERE OLD.a > 0 DO INSTEAD DELETE FROM base_tbl WHERE a=OLD.a;

INSERT INTO rw_view16 (a, b) VALUES (3, 'Row 3');

UPDATE rw_view16 SET b='ROW 2' WHERE a=2;

DELETE FROM rw_view16 WHERE a=2;

DROP TABLE base_tbl CASCADE;

DROP VIEW ro_view10, ro_view12, ro_view18;

DROP SEQUENCE uv_seq CASCADE;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW rw_view1 AS
  SELECT *, 'Const' AS c, (SELECT concat('b: ', b)) AS d FROM base_tbl WHERE a>0;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name = 'rw_view1';

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name = 'rw_view1';

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name = 'rw_view1'
 ORDER BY ordinal_position;

INSERT INTO rw_view1 VALUES (3, 'Row 3');

INSERT INTO rw_view1 (a) VALUES (4);

UPDATE rw_view1 SET a=5 WHERE a=4;

DELETE FROM rw_view1 WHERE b='Row 2';

SELECT * FROM base_tbl;

SET jit_above_cost = 0;

SET jit_above_cost TO DEFAULT;

SELECT * FROM base_tbl ORDER BY a;

SELECT * FROM base_tbl ORDER BY a;

UPDATE rw_view1 SET a=6 WHERE a=5;

DELETE FROM rw_view1 WHERE a=5;

CREATE TABLE base_tbl_hist(ts timestamptz default now(), a int, b text);

CREATE RULE base_tbl_log AS ON INSERT TO rw_view1 DO ALSO
  INSERT INTO base_tbl_hist(a,b) VALUES(new.a, new.b);

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name = 'rw_view1';

INSERT INTO rw_view1 VALUES (9, DEFAULT), (10, DEFAULT);

SELECT a, b FROM base_tbl_hist;

DROP TABLE base_tbl CASCADE;

DROP TABLE base_tbl_hist;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW rw_view1 AS
  SELECT b AS bb, a AS aa, 'Const1' AS c FROM base_tbl WHERE a>0;

CREATE VIEW rw_view2 AS
  SELECT aa AS aaa, bb AS bbb, c AS c1, 'Const2' AS c2 FROM rw_view1 WHERE aa<10;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name = 'rw_view2';

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name = 'rw_view2';

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name = 'rw_view2'
 ORDER BY ordinal_position;

INSERT INTO rw_view2 VALUES (3, 'Row 3');

INSERT INTO rw_view2 (aaa) VALUES (4);

SELECT * FROM rw_view2;

UPDATE rw_view2 SET bbb='Row 4' WHERE aaa=4;

DELETE FROM rw_view2 WHERE aaa=2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 ORDER BY aaa;

SELECT * FROM rw_view2 ORDER BY aaa;

UPDATE rw_view2 SET aaa=5 WHERE aaa=4;

DELETE FROM rw_view2 WHERE aaa=4;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WHERE a>0 OFFSET 0;

CREATE VIEW rw_view2 AS SELECT * FROM rw_view1 WHERE a<10;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE RULE rw_view1_ins_rule AS ON INSERT TO rw_view1
  DO INSTEAD INSERT INTO base_tbl VALUES (NEW.a, NEW.b) RETURNING *;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE RULE rw_view1_upd_rule AS ON UPDATE TO rw_view1
  DO INSTEAD UPDATE base_tbl SET b=NEW.b WHERE a=OLD.a RETURNING NEW.*;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE RULE rw_view1_del_rule AS ON DELETE TO rw_view1
  DO INSTEAD DELETE FROM base_tbl WHERE a=OLD.a RETURNING OLD.*;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

INSERT INTO rw_view2 VALUES (3, 'Row 3') RETURNING old.*, new.*;

UPDATE rw_view2 SET b='R3' WHERE a=3 RETURNING old.*, new.*;

DROP RULE rw_view1_upd_rule ON rw_view1;

CREATE RULE rw_view1_upd_rule AS ON UPDATE TO rw_view1
  DO INSTEAD UPDATE base_tbl SET b=NEW.b WHERE a=OLD.a RETURNING *;

UPDATE rw_view2 SET b='Row three' WHERE a=3 RETURNING old.*, new.*;

SELECT * FROM rw_view2;

DELETE FROM rw_view2 WHERE a=3 RETURNING old.*, new.*;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET a=3 WHERE a=2;

DELETE FROM rw_view2 WHERE a=2;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW rw_view1 AS
  SELECT *, 'Const1' AS c1 FROM base_tbl WHERE a>0 OFFSET 0;

CREATE VIEW rw_view2 AS
  SELECT *, 'Const2' AS c2 FROM rw_view1 WHERE a<10;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into,
       is_trigger_updatable, is_trigger_deletable,
       is_trigger_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE FUNCTION rw_view1_trig_fn()
RETURNS trigger AS
$$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO base_tbl VALUES (NEW.a, NEW.b);
    NEW.c1 = 'Trigger Const1';
    RETURN NEW;
  ELSIF TG_OP = 'UPDATE' THEN
    UPDATE base_tbl SET b=NEW.b WHERE a=OLD.a;
    NEW.c1 = 'Trigger Const1';
    RETURN NEW;
  ELSIF TG_OP = 'DELETE' THEN
    DELETE FROM base_tbl WHERE a=OLD.a;
    RETURN OLD;
  END IF;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER rw_view1_ins_trig INSTEAD OF INSERT ON rw_view1
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into,
       is_trigger_updatable, is_trigger_deletable,
       is_trigger_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE TRIGGER rw_view1_upd_trig INSTEAD OF UPDATE ON rw_view1
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into,
       is_trigger_updatable, is_trigger_deletable,
       is_trigger_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

CREATE TRIGGER rw_view1_del_trig INSTEAD OF DELETE ON rw_view1
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into,
       is_trigger_updatable, is_trigger_deletable,
       is_trigger_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE 'rw_view%'
 ORDER BY table_name, ordinal_position;

INSERT INTO rw_view2 VALUES (3, 'Row 3') RETURNING old.*, new.*;

UPDATE rw_view2 SET b='Row three' WHERE a=3 RETURNING old.*, new.*;

SELECT * FROM rw_view2;

DELETE FROM rw_view2 WHERE a=3 RETURNING old.*, new.*;

SELECT * FROM rw_view2;

SELECT * FROM base_tbl ORDER BY a;

SELECT * FROM base_tbl ORDER BY a;

UPDATE rw_view2 SET a=3 WHERE a=2;

DELETE FROM rw_view2 WHERE a=2;

DROP TRIGGER rw_view1_del_trig ON rw_view1;

DROP TRIGGER rw_view1_ins_trig ON rw_view1;

CREATE TRIGGER rw_view2_upd_trig INSTEAD OF UPDATE ON rw_view2
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

SELECT * FROM base_tbl ORDER BY a;

DROP TABLE base_tbl CASCADE;

DROP FUNCTION rw_view1_trig_fn();

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);

CREATE VIEW rw_view1 AS SELECT b AS bb, a AS aa FROM base_tbl;

CREATE FUNCTION rw_view1_aa(x rw_view1)
  RETURNS int AS $$ SELECT x.aa $$ LANGUAGE sql;

UPDATE rw_view1 v SET bb='Updated row 2' WHERE rw_view1_aa(v)=2
  RETURNING rw_view1_aa(v), v.bb;

SELECT * FROM base_tbl;

UPDATE rw_view1 v SET bb='Updated row 2' WHERE rw_view1_aa(v)=2
  RETURNING rw_view1_aa(v), v.bb;

DROP TABLE base_tbl CASCADE;

CREATE USER regress_view_user1;

CREATE USER regress_view_user2;

CREATE USER regress_view_user3;

SET SESSION AUTHORIZATION regress_view_user1;

CREATE TABLE base_tbl(a int, b text, c float);

INSERT INTO base_tbl VALUES (1, 'Row 1', 1.0);

CREATE VIEW rw_view1 AS SELECT b AS bb, c AS cc, a AS aa FROM base_tbl;

INSERT INTO rw_view1 VALUES ('Row 2', 2.0, 2);

GRANT SELECT ON base_tbl TO regress_view_user2;

GRANT SELECT ON rw_view1 TO regress_view_user2;

GRANT UPDATE (a,c) ON base_tbl TO regress_view_user2;

GRANT UPDATE (bb,cc) ON rw_view1 TO regress_view_user2;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_view_user2;

CREATE VIEW rw_view2 AS SELECT b AS bb, c AS cc, a AS aa FROM base_tbl;

SELECT * FROM base_tbl;

SELECT * FROM rw_view1;

SELECT * FROM rw_view2;

INSERT INTO base_tbl VALUES (3, 'Row 3', 3.0);

INSERT INTO rw_view1 VALUES ('Row 3', 3.0, 3);

INSERT INTO rw_view2 VALUES ('Row 3', 3.0, 3);

UPDATE base_tbl SET a=a, c=c;

UPDATE base_tbl SET b=b;

UPDATE rw_view1 SET bb=bb, cc=cc;

UPDATE rw_view1 SET aa=aa;

UPDATE rw_view2 SET aa=aa, cc=cc;

UPDATE rw_view2 SET bb=bb;

DELETE FROM base_tbl;

DELETE FROM rw_view1;

DELETE FROM rw_view2;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT INSERT, DELETE ON base_tbl TO regress_view_user2;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_view_user2;

INSERT INTO base_tbl VALUES (3, 'Row 3', 3.0);

INSERT INTO rw_view1 VALUES ('Row 4', 4.0, 4);

INSERT INTO rw_view2 VALUES ('Row 4', 4.0, 4);

DELETE FROM base_tbl WHERE a=1;

DELETE FROM rw_view1 WHERE aa=2;

DELETE FROM rw_view2 WHERE aa=2;

SELECT * FROM base_tbl;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_view_user1;

REVOKE INSERT, DELETE ON base_tbl FROM regress_view_user2;

GRANT INSERT, DELETE ON rw_view1 TO regress_view_user2;

RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION regress_view_user2;

INSERT INTO base_tbl VALUES (5, 'Row 5', 5.0);

INSERT INTO rw_view1 VALUES ('Row 5', 5.0, 5);

INSERT INTO rw_view2 VALUES ('Row 6', 6.0, 6);

DELETE FROM base_tbl WHERE a=3;

DELETE FROM rw_view1 WHERE aa=3;

DELETE FROM rw_view2 WHERE aa=4;

SELECT * FROM base_tbl;

RESET SESSION AUTHORIZATION;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl(a int, b text, c float);

INSERT INTO base_tbl VALUES (1, 'Row 1', 1.0);

SET SESSION AUTHORIZATION regress_view_user1;

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl;

SELECT * FROM rw_view1;

SELECT * FROM rw_view1 FOR UPDATE;

UPDATE rw_view1 SET b = 'foo' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user2;

CREATE VIEW rw_view2 AS SELECT * FROM rw_view1;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

RESET SESSION AUTHORIZATION;

GRANT SELECT ON base_tbl TO regress_view_user1;

SET SESSION AUTHORIZATION regress_view_user1;

SELECT * FROM rw_view1;

SELECT * FROM rw_view1 FOR UPDATE;

UPDATE rw_view1 SET b = 'foo' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT SELECT ON rw_view1 TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

RESET SESSION AUTHORIZATION;

GRANT UPDATE ON base_tbl TO regress_view_user1;

SET SESSION AUTHORIZATION regress_view_user1;

SELECT * FROM rw_view1;

SELECT * FROM rw_view1 FOR UPDATE;

UPDATE rw_view1 SET b = 'foo' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT UPDATE ON rw_view1 TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

RESET SESSION AUTHORIZATION;

REVOKE UPDATE ON base_tbl FROM regress_view_user1;

SET SESSION AUTHORIZATION regress_view_user1;

SELECT * FROM rw_view1;

SELECT * FROM rw_view1 FOR UPDATE;

UPDATE rw_view1 SET b = 'foo' WHERE a = 1;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

SELECT * FROM rw_view2 FOR UPDATE;

UPDATE rw_view2 SET b = 'bar' WHERE a = 1;

RESET SESSION AUTHORIZATION;

DROP TABLE base_tbl CASCADE;

SET SESSION AUTHORIZATION regress_view_user1;

CREATE TABLE base_tbl(a int, b text, c float);

INSERT INTO base_tbl VALUES (1, 'Row 1', 1.0);

CREATE VIEW rw_view1 AS SELECT b AS bb, c AS cc, a AS aa FROM base_tbl;

ALTER VIEW rw_view1 SET (security_invoker = true);

INSERT INTO rw_view1 VALUES ('Row 2', 2.0, 2);

GRANT SELECT ON rw_view1 TO regress_view_user2;

GRANT UPDATE (bb,cc) ON rw_view1 TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM base_tbl;

SELECT * FROM rw_view1;

INSERT INTO base_tbl VALUES (3, 'Row 3', 3.0);

INSERT INTO rw_view1 VALUES ('Row 3', 3.0, 3);

UPDATE base_tbl SET a=a;

UPDATE rw_view1 SET bb=bb, cc=cc;

DELETE FROM base_tbl;

DELETE FROM rw_view1;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT SELECT ON base_tbl TO regress_view_user2;

GRANT UPDATE (a,c) ON base_tbl TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM base_tbl;

SELECT * FROM rw_view1;

UPDATE base_tbl SET a=a, c=c;

UPDATE base_tbl SET b=b;

UPDATE rw_view1 SET cc=cc;

UPDATE rw_view1 SET aa=aa;

UPDATE rw_view1 SET bb=bb;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT INSERT, DELETE ON base_tbl TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

INSERT INTO base_tbl VALUES (3, 'Row 3', 3.0);

INSERT INTO rw_view1 VALUES ('Row 4', 4.0, 4);

DELETE FROM base_tbl WHERE a=1;

DELETE FROM rw_view1 WHERE aa=2;

SET SESSION AUTHORIZATION regress_view_user1;

REVOKE INSERT, DELETE ON base_tbl FROM regress_view_user2;

GRANT INSERT, DELETE ON rw_view1 TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

INSERT INTO rw_view1 VALUES ('Row 4', 4.0, 4);

DELETE FROM rw_view1 WHERE aa=2;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT INSERT, DELETE ON base_tbl TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

INSERT INTO rw_view1 VALUES ('Row 4', 4.0, 4);

DELETE FROM rw_view1 WHERE aa=2;

SELECT * FROM base_tbl;

RESET SESSION AUTHORIZATION;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl(a int, b text, c float);

INSERT INTO base_tbl VALUES (1, 'Row 1', 1.0);

SET SESSION AUTHORIZATION regress_view_user1;

CREATE VIEW rw_view1 AS SELECT b AS bb, c AS cc, a AS aa FROM base_tbl;

ALTER VIEW rw_view1 SET (security_invoker = true);

SELECT * FROM rw_view1;

UPDATE rw_view1 SET aa=aa;

SET SESSION AUTHORIZATION regress_view_user2;

CREATE VIEW rw_view2 AS SELECT cc AS ccc, aa AS aaa, bb AS bbb FROM rw_view1;

GRANT SELECT, UPDATE ON rw_view2 TO regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

RESET SESSION AUTHORIZATION;

GRANT SELECT ON base_tbl TO regress_view_user1;

GRANT UPDATE (a, b) ON base_tbl TO regress_view_user1;

SET SESSION AUTHORIZATION regress_view_user1;

SELECT * FROM rw_view1;

UPDATE rw_view1 SET aa=aa, bb=bb;

UPDATE rw_view1 SET cc=cc;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

SET SESSION AUTHORIZATION regress_view_user1;

GRANT SELECT ON rw_view1 TO regress_view_user2;

GRANT UPDATE (bb, cc) ON rw_view1 TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET bbb=bbb;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET bbb=bbb;

RESET SESSION AUTHORIZATION;

GRANT SELECT ON base_tbl TO regress_view_user2;

GRANT UPDATE (a, c) ON base_tbl TO regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

RESET SESSION AUTHORIZATION;

GRANT SELECT ON base_tbl TO regress_view_user3;

GRANT UPDATE (a, c) ON base_tbl TO regress_view_user3;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

RESET SESSION AUTHORIZATION;

REVOKE SELECT, UPDATE ON base_tbl FROM regress_view_user1;

SET SESSION AUTHORIZATION regress_view_user1;

SELECT * FROM rw_view1;

UPDATE rw_view1 SET aa=aa;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

RESET SESSION AUTHORIZATION;

REVOKE SELECT, UPDATE ON base_tbl FROM regress_view_user2;

SET SESSION AUTHORIZATION regress_view_user2;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

SET SESSION AUTHORIZATION regress_view_user3;

SELECT * FROM rw_view2;

UPDATE rw_view2 SET aaa=aaa;

UPDATE rw_view2 SET bbb=bbb;

UPDATE rw_view2 SET ccc=ccc;

RESET SESSION AUTHORIZATION;

DROP TABLE base_tbl CASCADE;

DROP USER regress_view_user1;

DROP USER regress_view_user2;

DROP USER regress_view_user3;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified', c serial);

INSERT INTO base_tbl VALUES (1, 'Row 1');

INSERT INTO base_tbl VALUES (2, 'Row 2');

INSERT INTO base_tbl VALUES (3);

CREATE VIEW rw_view1 AS SELECT a AS aa, b AS bb FROM base_tbl;

ALTER VIEW rw_view1 ALTER COLUMN bb SET DEFAULT 'View default';

INSERT INTO rw_view1 VALUES (4, 'Row 4');

INSERT INTO rw_view1 (aa) VALUES (5);

SELECT * FROM base_tbl;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int PRIMARY KEY, b text DEFAULT 'Unspecified');

INSERT INTO base_tbl VALUES (1, 'Row 1');

INSERT INTO base_tbl VALUES (2, 'Row 2');

CREATE FUNCTION rw_view1_trig_fn()
RETURNS trigger AS
$$
BEGIN
  IF TG_OP = 'INSERT' THEN
    UPDATE base_tbl SET b=NEW.b WHERE a=1;
    RETURN NULL;
  END IF;
  RETURN NULL;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER rw_view1_ins_trig AFTER INSERT ON base_tbl
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

CREATE VIEW rw_view1 AS SELECT a AS aa, b AS bb FROM base_tbl;

INSERT INTO rw_view1 VALUES (3, 'Row 3');

select * from base_tbl;

DROP VIEW rw_view1;

DROP TRIGGER rw_view1_ins_trig on base_tbl;

DROP FUNCTION rw_view1_trig_fn();

DROP TABLE base_tbl;

CREATE TABLE base_tbl (a int, b int);

INSERT INTO base_tbl VALUES (1,2), (4,5), (3,-3);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl ORDER BY a+b;

SELECT * FROM rw_view1;

INSERT INTO rw_view1 VALUES (7,-8);

SELECT * FROM rw_view1;

UPDATE rw_view1 SET b = b + 1 RETURNING *;

UPDATE rw_view1 SET b = b + 1 RETURNING *;

SELECT * FROM rw_view1;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int, arr int[]);

INSERT INTO base_tbl VALUES (1,ARRAY[2]), (3,ARRAY[4]);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl;

UPDATE rw_view1 SET arr[1] = 42, arr[2] = 77 WHERE a = 3;

SELECT * FROM rw_view1;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl(a float);

INSERT INTO base_tbl SELECT i/10.0 FROM generate_series(1,10) g(i);

CREATE VIEW rw_view1 AS
  SELECT ctid, sin(a) s, a, cos(a) c
  FROM base_tbl
  WHERE a != 0
  ORDER BY abs(a);

INSERT INTO rw_view1 VALUES (null, null, 1.1, null);

INSERT INTO rw_view1 (s, c, a) VALUES (null, null, 1.1);

INSERT INTO rw_view1 (s, c, a) VALUES (default, default, 1.1);

INSERT INTO rw_view1 (a) VALUES (1.1) RETURNING a, s, c;

UPDATE rw_view1 SET s = s WHERE a = 1.1;

UPDATE rw_view1 SET a = 1.05 WHERE a = 1.1 RETURNING s;

DELETE FROM rw_view1 WHERE a = 1.05;

CREATE VIEW rw_view2 AS
  SELECT s, c, s/c t, a base_a, ctid
  FROM rw_view1;

INSERT INTO rw_view2 VALUES (null, null, null, 1.1, null);

INSERT INTO rw_view2(s, c, base_a) VALUES (null, null, 1.1);

INSERT INTO rw_view2(base_a) VALUES (1.1) RETURNING t;

UPDATE rw_view2 SET s = s WHERE base_a = 1.1;

UPDATE rw_view2 SET t = t WHERE base_a = 1.1;

UPDATE rw_view2 SET base_a = 1.05 WHERE base_a = 1.1;

DELETE FROM rw_view2 WHERE base_a = 1.05 RETURNING base_a, s, c, t;

CREATE VIEW rw_view3 AS
  SELECT s, c, s/c t, ctid
  FROM rw_view1;

INSERT INTO rw_view3 VALUES (null, null, null, null);

INSERT INTO rw_view3(s) VALUES (null);

UPDATE rw_view3 SET s = s;

DELETE FROM rw_view3 WHERE s = sin(0.1);

SELECT * FROM base_tbl ORDER BY a;

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name;

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name;

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name LIKE E'r_\\_view%'
 ORDER BY table_name, ordinal_position;

SELECT events & 4 != 0 AS upd,
       events & 8 != 0 AS ins,
       events & 16 != 0 AS del
  FROM pg_catalog.pg_relation_is_updatable('rw_view3'::regclass, false) t(events);

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (id int, idplus1 int GENERATED ALWAYS AS (id + 1) STORED);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl;

INSERT INTO base_tbl (id) VALUES (1);

INSERT INTO rw_view1 (id) VALUES (2);

INSERT INTO base_tbl (id, idplus1) VALUES (3, DEFAULT);

INSERT INTO rw_view1 (id, idplus1) VALUES (4, DEFAULT);

INSERT INTO base_tbl (id, idplus1) VALUES (5, 6);

INSERT INTO rw_view1 (id, idplus1) VALUES (6, 7);

SELECT * FROM base_tbl;

UPDATE base_tbl SET id = 2000 WHERE id = 2;

UPDATE rw_view1 SET id = 3000 WHERE id = 3;

SELECT * FROM base_tbl;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl_parent (a int);

CREATE TABLE base_tbl_child (CHECK (a > 0)) INHERITS (base_tbl_parent);

INSERT INTO base_tbl_parent SELECT * FROM generate_series(-8, -1);

INSERT INTO base_tbl_child SELECT * FROM generate_series(1, 8);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl_parent;

CREATE VIEW rw_view2 AS SELECT * FROM ONLY base_tbl_parent;

SELECT * FROM rw_view1 ORDER BY a;

SELECT * FROM ONLY rw_view1 ORDER BY a;

SELECT * FROM rw_view2 ORDER BY a;

INSERT INTO rw_view1 VALUES (-100), (100);

INSERT INTO rw_view2 VALUES (-200), (200);

UPDATE rw_view1 SET a = a*10 WHERE a IN (-1, 1);

UPDATE ONLY rw_view1 SET a = a*10 WHERE a IN (-2, 2);

UPDATE rw_view2 SET a = a*10 WHERE a IN (-3, 3);

UPDATE ONLY rw_view2 SET a = a*10 WHERE a IN (-4, 4);

DELETE FROM rw_view1 WHERE a IN (-5, 5);

DELETE FROM ONLY rw_view1 WHERE a IN (-6, 6);

DELETE FROM rw_view2 WHERE a IN (-7, 7);

DELETE FROM ONLY rw_view2 WHERE a IN (-8, 8);

SELECT * FROM ONLY base_tbl_parent ORDER BY a;

SELECT * FROM base_tbl_child ORDER BY a;

SELECT * FROM ONLY base_tbl_parent ORDER BY a;

SELECT * FROM base_tbl_child ORDER BY a;

CREATE TABLE other_tbl_parent (id int);

CREATE TABLE other_tbl_child () INHERITS (other_tbl_parent);

INSERT INTO other_tbl_parent VALUES (7),(200);

INSERT INTO other_tbl_child VALUES (8),(100);

UPDATE rw_view1 SET a = a + 1000 FROM other_tbl_parent WHERE a = id;

UPDATE rw_view1 SET a = a + 1000 FROM other_tbl_parent WHERE a = id;

SELECT * FROM ONLY base_tbl_parent ORDER BY a;

SELECT * FROM base_tbl_child ORDER BY a;

DROP TABLE base_tbl_parent, base_tbl_child CASCADE;

DROP TABLE other_tbl_parent CASCADE;

CREATE TABLE base_tbl (a int, b int DEFAULT 10);

INSERT INTO base_tbl VALUES (1,2), (2,3), (1,-1);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WHERE a < b
  WITH LOCAL CHECK OPTION;

SELECT * FROM information_schema.views WHERE table_name = 'rw_view1';

INSERT INTO rw_view1 VALUES(3,4);

INSERT INTO rw_view1 VALUES(4,3);

INSERT INTO rw_view1 VALUES(5,null);

UPDATE rw_view1 SET b = 5 WHERE a = 3;

UPDATE rw_view1 SET b = -5 WHERE a = 3;

INSERT INTO rw_view1(a) VALUES (9);

INSERT INTO rw_view1(a) VALUES (10);

SELECT * FROM base_tbl ORDER BY a, b;

SELECT * FROM base_tbl ORDER BY a, b;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WHERE a > 0;

CREATE VIEW rw_view2 AS SELECT * FROM rw_view1 WHERE a < 10
  WITH CHECK OPTION;

SELECT * FROM information_schema.views WHERE table_name = 'rw_view2';

INSERT INTO rw_view2 VALUES (-5);

INSERT INTO rw_view2 VALUES (5);

INSERT INTO rw_view2 VALUES (15);

SELECT * FROM base_tbl;

UPDATE rw_view2 SET a = a - 10;

UPDATE rw_view2 SET a = a + 10;

CREATE OR REPLACE VIEW rw_view2 AS SELECT * FROM rw_view1 WHERE a < 10
  WITH LOCAL CHECK OPTION;

SELECT * FROM information_schema.views WHERE table_name = 'rw_view2';

INSERT INTO rw_view2 VALUES (-10);

INSERT INTO rw_view2 VALUES (20);

SELECT * FROM base_tbl;

ALTER VIEW rw_view1 SET (check_option=here);

ALTER VIEW rw_view1 SET (check_option=local);

INSERT INTO rw_view2 VALUES (-20);

INSERT INTO rw_view2 VALUES (30);

ALTER VIEW rw_view2 RESET (check_option);

SELECT * FROM information_schema.views WHERE table_name = 'rw_view2';

INSERT INTO rw_view2 VALUES (30);

SELECT * FROM base_tbl;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WITH CHECK OPTION;

CREATE VIEW rw_view2 AS SELECT * FROM rw_view1 WHERE a > 0;

CREATE VIEW rw_view3 AS SELECT * FROM rw_view2 WITH CHECK OPTION;

SELECT * FROM information_schema.views WHERE table_name LIKE E'rw\\_view_' ORDER BY table_name;

INSERT INTO rw_view1 VALUES (-1);

INSERT INTO rw_view1 VALUES (1);

INSERT INTO rw_view2 VALUES (-2);

INSERT INTO rw_view2 VALUES (2);

INSERT INTO rw_view3 VALUES (-3);

INSERT INTO rw_view3 VALUES (3);

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int, b int[]);

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WHERE a = ANY (b)
  WITH CHECK OPTION;

INSERT INTO rw_view1 VALUES (1, ARRAY[1,2,3]);

INSERT INTO rw_view1 VALUES (10, ARRAY[4,5]);

UPDATE rw_view1 SET b[2] = -b[2] WHERE a = 1;

UPDATE rw_view1 SET b[1] = -b[1] WHERE a = 1;

INSERT INTO rw_view1 VALUES($1, $2);

EXECUTE ins(2, ARRAY[1,2,3]);

EXECUTE ins(10, ARRAY[4,5]);

DEALLOCATE PREPARE ins;

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (a int);

CREATE TABLE ref_tbl (a int PRIMARY KEY);

INSERT INTO ref_tbl SELECT * FROM generate_series(1,10);

CREATE VIEW rw_view1 AS
  SELECT * FROM base_tbl b
  WHERE EXISTS(SELECT 1 FROM ref_tbl r WHERE r.a = b.a)
  WITH CHECK OPTION;

INSERT INTO rw_view1 VALUES (5);

INSERT INTO rw_view1 VALUES (15);

UPDATE rw_view1 SET a = a + 5;

UPDATE rw_view1 SET a = a + 5;

INSERT INTO rw_view1 VALUES (5);

UPDATE rw_view1 SET a = a + 5;

DROP TABLE base_tbl, ref_tbl CASCADE;

CREATE TABLE base_tbl (a int, b int);

CREATE FUNCTION base_tbl_trig_fn()
RETURNS trigger AS
$$
BEGIN
  NEW.b := 10;
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER base_tbl_trig BEFORE INSERT OR UPDATE ON base_tbl
  FOR EACH ROW EXECUTE PROCEDURE base_tbl_trig_fn();

CREATE VIEW rw_view1 AS SELECT * FROM base_tbl WHERE a < b WITH CHECK OPTION;

INSERT INTO rw_view1 VALUES (5,0);

INSERT INTO rw_view1 VALUES (15, 20);

UPDATE rw_view1 SET a = 20, b = 30;

DROP TABLE base_tbl CASCADE;

DROP FUNCTION base_tbl_trig_fn();

CREATE TABLE base_tbl (a int, b int);

CREATE VIEW rw_view1 AS SELECT a FROM base_tbl WHERE a < b;

CREATE FUNCTION rw_view1_trig_fn()
RETURNS trigger AS
$$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO base_tbl VALUES (NEW.a, 10);
    RETURN NEW;
  ELSIF TG_OP = 'UPDATE' THEN
    UPDATE base_tbl SET a=NEW.a WHERE a=OLD.a;
    RETURN NEW;
  ELSIF TG_OP = 'DELETE' THEN
    DELETE FROM base_tbl WHERE a=OLD.a;
    RETURN OLD;
  END IF;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER rw_view1_trig
  INSTEAD OF INSERT OR UPDATE OR DELETE ON rw_view1
  FOR EACH ROW EXECUTE PROCEDURE rw_view1_trig_fn();

CREATE VIEW rw_view2 AS
  SELECT * FROM rw_view1 WHERE a > 0 WITH LOCAL CHECK OPTION;

INSERT INTO rw_view2 VALUES (-5);

INSERT INTO rw_view2 VALUES (5);

INSERT INTO rw_view2 VALUES (50);

UPDATE rw_view2 SET a = a - 10;

SELECT * FROM base_tbl;

ALTER VIEW rw_view2 SET (check_option=cascaded);

INSERT INTO rw_view2 VALUES (100);

UPDATE rw_view2 SET a = 200 WHERE a = 5;

SELECT * FROM base_tbl;

DROP TRIGGER rw_view1_trig ON rw_view1;

CREATE RULE rw_view1_ins_rule AS ON INSERT TO rw_view1
  DO INSTEAD INSERT INTO base_tbl VALUES (NEW.a, 10);

CREATE RULE rw_view1_upd_rule AS ON UPDATE TO rw_view1
  DO INSTEAD UPDATE base_tbl SET a=NEW.a WHERE a=OLD.a;

INSERT INTO rw_view2 VALUES (-10);

INSERT INTO rw_view2 VALUES (5);

INSERT INTO rw_view2 VALUES (20);

UPDATE rw_view2 SET a = 30 WHERE a = 5;

INSERT INTO rw_view2 VALUES (5);

UPDATE rw_view2 SET a = -5 WHERE a = 5;

SELECT * FROM base_tbl;

DROP TABLE base_tbl CASCADE;

DROP FUNCTION rw_view1_trig_fn();

CREATE TABLE base_tbl (a int);

CREATE VIEW rw_view1 AS SELECT a,10 AS b FROM base_tbl;

CREATE RULE rw_view1_ins_rule AS ON INSERT TO rw_view1
  DO INSTEAD INSERT INTO base_tbl VALUES (NEW.a);

CREATE VIEW rw_view2 AS
  SELECT * FROM rw_view1 WHERE a > b WITH LOCAL CHECK OPTION;

INSERT INTO rw_view2 VALUES (2,3);

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl (person text, visibility text);

INSERT INTO base_tbl VALUES ('Tom', 'public'),
                            ('Dick', 'private'),
                            ('Harry', 'public');

CREATE VIEW rw_view1 AS
  SELECT person FROM base_tbl WHERE visibility = 'public';

CREATE FUNCTION snoop(anyelement)
RETURNS boolean AS
$$
BEGIN
  RAISE NOTICE 'snooped value: %', $1;
  RETURN true;
END;
$$
LANGUAGE plpgsql COST 0.000001;

CREATE OR REPLACE FUNCTION leakproof(anyelement)
RETURNS boolean AS
$$
BEGIN
  RETURN true;
END;
$$
LANGUAGE plpgsql STRICT IMMUTABLE LEAKPROOF;

SELECT * FROM rw_view1 WHERE snoop(person);

UPDATE rw_view1 SET person=person WHERE snoop(person);

DELETE FROM rw_view1 WHERE NOT snoop(person);

ALTER VIEW rw_view1 SET (security_barrier = true);

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name = 'rw_view1';

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name = 'rw_view1';

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name = 'rw_view1'
 ORDER BY ordinal_position;

SELECT * FROM rw_view1 WHERE snoop(person);

UPDATE rw_view1 SET person=person WHERE snoop(person);

DELETE FROM rw_view1 WHERE NOT snoop(person);

SELECT * FROM rw_view1 WHERE snoop(person);

UPDATE rw_view1 SET person=person WHERE snoop(person);

DELETE FROM rw_view1 WHERE NOT snoop(person);

CREATE VIEW rw_view2 WITH (security_barrier = true) AS
  SELECT * FROM rw_view1 WHERE snoop(person);

SELECT table_name, is_insertable_into
  FROM information_schema.tables
 WHERE table_name = 'rw_view2';

SELECT table_name, is_updatable, is_insertable_into
  FROM information_schema.views
 WHERE table_name = 'rw_view2';

SELECT table_name, column_name, is_updatable
  FROM information_schema.columns
 WHERE table_name = 'rw_view2'
 ORDER BY ordinal_position;

SELECT * FROM rw_view2 WHERE snoop(person);

UPDATE rw_view2 SET person=person WHERE snoop(person);

DELETE FROM rw_view2 WHERE NOT snoop(person);

SELECT * FROM rw_view2 WHERE snoop(person);

UPDATE rw_view2 SET person=person WHERE snoop(person);

DELETE FROM rw_view2 WHERE NOT snoop(person);

DROP TABLE base_tbl CASCADE;

CREATE TABLE base_tbl(id int PRIMARY KEY, data text, deleted boolean);

INSERT INTO base_tbl VALUES (1, 'Row 1', false), (2, 'Row 2', true);

CREATE RULE base_tbl_ins_rule AS ON INSERT TO base_tbl
  WHERE EXISTS (SELECT 1 FROM base_tbl t WHERE t.id = new.id)
  DO INSTEAD
    UPDATE base_tbl SET data = new.data, deleted = false WHERE id = new.id;

CREATE RULE base_tbl_del_rule AS ON DELETE TO base_tbl
  DO INSTEAD
    UPDATE base_tbl SET deleted = true WHERE id = old.id;

CREATE VIEW rw_view1 WITH (security_barrier=true) AS
  SELECT id, data FROM base_tbl WHERE NOT deleted;

SELECT * FROM rw_view1;

DELETE FROM rw_view1 WHERE id = 1 AND snoop(data);

DELETE FROM rw_view1 WHERE id = 1 AND snoop(data);

INSERT INTO rw_view1 VALUES (2, 'New row 2');

INSERT INTO rw_view1 VALUES (2, 'New row 2');

SELECT * FROM base_tbl;

DROP TABLE base_tbl CASCADE;

CREATE TABLE t1 (a int, b float, c text);

CREATE INDEX t1_a_idx ON t1(a);

INSERT INTO t1
SELECT i,i,'t1' FROM generate_series(1,10) g(i);

ANALYZE t1;

CREATE TABLE t11 (d text) INHERITS (t1);

CREATE INDEX t11_a_idx ON t11(a);

INSERT INTO t11
SELECT i,i,'t11','t11d' FROM generate_series(1,10) g(i);

ANALYZE t11;

CREATE TABLE t12 (e int[]) INHERITS (t1);

CREATE INDEX t12_a_idx ON t12(a);

INSERT INTO t12
SELECT i,i,'t12','{1,2}'::int[] FROM generate_series(1,10) g(i);

ANALYZE t12;

CREATE TABLE t111 () INHERITS (t11, t12);

CREATE INDEX t111_a_idx ON t111(a);

INSERT INTO t111
SELECT i,i,'t111','t111d','{1,1,1}'::int[] FROM generate_series(1,10) g(i);

ANALYZE t111;

CREATE VIEW v1 WITH (security_barrier=true) AS
SELECT *, (SELECT d FROM t11 WHERE t11.a = t1.a LIMIT 1) AS d
FROM t1
WHERE a > 5 AND EXISTS(SELECT 1 FROM t12 WHERE t12.a = t1.a);

SELECT * FROM v1 WHERE a=3;

SELECT * FROM v1 WHERE a=8;

UPDATE v1 SET a=100 WHERE snoop(a) AND leakproof(a) AND a < 7 AND a != 6;

UPDATE v1 SET a=100 WHERE snoop(a) AND leakproof(a) AND a < 7 AND a != 6;

SELECT * FROM v1 WHERE a=100;

SELECT * FROM t1 WHERE a=100;

UPDATE v1 SET a=a+1 WHERE snoop(a) AND leakproof(a) AND a = 8;

UPDATE v1 SET a=a+1 WHERE snoop(a) AND leakproof(a) AND a = 8;

SELECT * FROM v1 WHERE b=8;

DELETE FROM v1 WHERE snoop(a) AND leakproof(a);

TABLE t1;

DROP TABLE t1, t11, t12, t111 CASCADE;

DROP FUNCTION snoop(anyelement);

DROP FUNCTION leakproof(anyelement);

CREATE TABLE tx1 (a integer);

CREATE TABLE tx2 (b integer);

CREATE TABLE tx3 (c integer);

CREATE VIEW vx1 AS SELECT a FROM tx1 WHERE EXISTS(SELECT 1 FROM tx2 JOIN tx3 ON b=c);

INSERT INTO vx1 values (1);

SELECT * FROM tx1;

SELECT * FROM vx1;

DROP VIEW vx1;

DROP TABLE tx1;

DROP TABLE tx2;

DROP TABLE tx3;

CREATE TABLE tx1 (a integer);

CREATE TABLE tx2 (b integer);

CREATE TABLE tx3 (c integer);

CREATE VIEW vx1 AS SELECT a FROM tx1 WHERE EXISTS(SELECT 1 FROM tx2 JOIN tx3 ON b=c);

INSERT INTO vx1 VALUES (1);

INSERT INTO vx1 VALUES (1);

SELECT * FROM tx1;

SELECT * FROM vx1;

DROP VIEW vx1;

DROP TABLE tx1;

DROP TABLE tx2;

DROP TABLE tx3;

CREATE TABLE tx1 (a integer, b integer);

CREATE TABLE tx2 (b integer, c integer);

CREATE TABLE tx3 (c integer, d integer);

ALTER TABLE tx1 DROP COLUMN b;

ALTER TABLE tx2 DROP COLUMN c;

ALTER TABLE tx3 DROP COLUMN d;

CREATE VIEW vx1 AS SELECT a FROM tx1 WHERE EXISTS(SELECT 1 FROM tx2 JOIN tx3 ON b=c);

INSERT INTO vx1 VALUES (1);

INSERT INTO vx1 VALUES (1);

SELECT * FROM tx1;

SELECT * FROM vx1;

DROP VIEW vx1;

DROP TABLE tx1;

DROP TABLE tx2;

DROP TABLE tx3;

CREATE TABLE t1 (a int, b text, c int);

INSERT INTO t1 VALUES (1, 'one', 10);

CREATE TABLE t2 (cc int);

INSERT INTO t2 VALUES (10), (20);

CREATE VIEW v1 WITH (security_barrier = true) AS
  SELECT * FROM t1 WHERE (a > 0)
  WITH CHECK OPTION;

CREATE VIEW v2 WITH (security_barrier = true) AS
  SELECT * FROM v1 WHERE EXISTS (SELECT 1 FROM t2 WHERE t2.cc = v1.c)
  WITH CHECK OPTION;

INSERT INTO v2 VALUES (2, 'two', 20);

INSERT INTO v2 VALUES (-2, 'minus two', 20);

INSERT INTO v2 VALUES (3, 'three', 30);

UPDATE v2 SET b = 'ONE' WHERE a = 1;

UPDATE v2 SET a = -1 WHERE a = 1;

UPDATE v2 SET c = 30 WHERE a = 1;

DELETE FROM v2 WHERE a = 2;

SELECT * FROM v2;

DROP VIEW v2;

DROP VIEW v1;

DROP TABLE t2;

DROP TABLE t1;

CREATE TABLE t1 (a int);

CREATE VIEW v1 WITH (security_barrier = true) AS
  SELECT * FROM t1;

CREATE RULE v1_upd_rule AS ON UPDATE TO v1 DO INSTEAD
  UPDATE t1 SET a = NEW.a WHERE a = OLD.a;

CREATE VIEW v2 WITH (security_barrier = true) AS
  SELECT * FROM v1 WHERE EXISTS (SELECT 1);

UPDATE v2 SET a = 1;

DROP VIEW v2;

DROP VIEW v1;

DROP TABLE t1;

CREATE TABLE t1 (a int, b text);

CREATE VIEW v1 AS SELECT null::int AS a;

CREATE OR REPLACE VIEW v1 AS SELECT * FROM t1 WHERE a > 0 WITH CHECK OPTION;

INSERT INTO v1 VALUES (1, 'ok');

INSERT INTO v1 VALUES (-1, 'invalid');

DROP VIEW v1;

DROP TABLE t1;

create table uv_pt (a int, b int, v varchar) partition by range (a, b);

create table uv_pt1 (b int not null, v varchar, a int not null) partition by range (b);

create table uv_pt11 (like uv_pt1);

alter table uv_pt11 drop a;

alter table uv_pt11 add a int;

alter table uv_pt11 drop a;

alter table uv_pt11 add a int not null;

alter table uv_pt1 attach partition uv_pt11 for values from (2) to (5);

alter table uv_pt attach partition uv_pt1 for values from (1, 2) to (1, 10);

create view uv_ptv as select * from uv_pt;

select events & 4 != 0 AS upd,
       events & 8 != 0 AS ins,
       events & 16 != 0 AS del
  from pg_catalog.pg_relation_is_updatable('uv_pt'::regclass, false) t(events);

select pg_catalog.pg_column_is_updatable('uv_pt'::regclass, 1::smallint, false);

select pg_catalog.pg_column_is_updatable('uv_pt'::regclass, 2::smallint, false);

select table_name, is_updatable, is_insertable_into
  from information_schema.views where table_name = 'uv_ptv';

select table_name, column_name, is_updatable
  from information_schema.columns where table_name = 'uv_ptv' order by column_name;

insert into uv_ptv values (1, 2);

select tableoid::regclass, * from uv_pt;

create view uv_ptv_wco as select * from uv_pt where a = 0 with check option;

insert into uv_ptv_wco values (1, 2);

select tableoid::regclass, * from uv_pt order by a, b;

drop view uv_ptv, uv_ptv_wco;

drop table uv_pt, uv_pt1, uv_pt11;

create table wcowrtest (a int) partition by list (a);

create table wcowrtest1 partition of wcowrtest for values in (1);

create view wcowrtest_v as select * from wcowrtest where wcowrtest = '(2)'::wcowrtest with check option;

insert into wcowrtest_v values (1);

alter table wcowrtest add b text;

create table wcowrtest2 (b text, c int, a int);

alter table wcowrtest2 drop c;

alter table wcowrtest attach partition wcowrtest2 for values in (2);

create table sometable (a int, b text);

insert into sometable values (1, 'a'), (2, 'b');

create view wcowrtest_v2 as
    select *
      from wcowrtest r
      where r in (select s from sometable s where r.a = s.a)
with check option;

insert into wcowrtest_v2 values (2, 'no such row in sometable');

drop view wcowrtest_v, wcowrtest_v2;

drop table wcowrtest, sometable;

create table uv_iocu_tab (a text unique, b float);

insert into uv_iocu_tab values ('xyxyxy', 0);

create view uv_iocu_view as
   select b, b+1 as c, a, '2.0'::text as two from uv_iocu_tab;

insert into uv_iocu_view (a, b) values ('xyxyxy', 1)
   on conflict (a) do update set b = uv_iocu_view.b;

select * from uv_iocu_tab;

insert into uv_iocu_view (a, b) values ('xyxyxy', 1)
   on conflict (a) do update set b = excluded.b;

select * from uv_iocu_tab;

insert into uv_iocu_view (a, b) values ('xyxyxy', 3)
   on conflict (a) do update set b = cast(excluded.two as float);

select * from uv_iocu_tab;

insert into uv_iocu_view (a, b) values ('xyxyxy', 3)
   on conflict (a) do update set b = excluded.b where excluded.c > 0;

insert into uv_iocu_view (a, b) values ('xyxyxy', 3)
   on conflict (a) do update set b = excluded.b where excluded.c > 0;

select * from uv_iocu_tab;

drop view uv_iocu_view;

drop table uv_iocu_tab;

create table uv_iocu_tab (a int unique, b text);

create view uv_iocu_view as
    select b as bb, a as aa, uv_iocu_tab::text as cc from uv_iocu_tab;

insert into uv_iocu_view (aa,bb) values (1,'x');

insert into uv_iocu_view (aa,bb) values (1,'y')
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*
   where excluded.aa > 0
   and excluded.bb != ''
   and excluded.cc is not null;

insert into uv_iocu_view (aa,bb) values (1,'y')
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*
   where excluded.aa > 0
   and excluded.bb != ''
   and excluded.cc is not null;

select * from uv_iocu_view;

delete from uv_iocu_view;

insert into uv_iocu_view (aa,bb) values (1,'x');

insert into uv_iocu_view (aa) values (1)
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*;

select * from uv_iocu_view;

alter table uv_iocu_tab alter column b set default 'table default';

insert into uv_iocu_view (aa) values (1)
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*;

select * from uv_iocu_view;

alter view uv_iocu_view alter column bb set default 'view default';

insert into uv_iocu_view (aa) values (1)
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*;

select * from uv_iocu_view;

insert into uv_iocu_view (aa) values (1)
   on conflict (aa) do update set cc = 'XXX';

drop view uv_iocu_view;

drop table uv_iocu_tab;

create user regress_view_user1;

create user regress_view_user2;

set session authorization regress_view_user1;

create table base_tbl(a int unique, b text, c float);

insert into base_tbl values (1,'xxx',1.0);

create view rw_view1 as select b as bb, c as cc, a as aa from base_tbl;

grant select (aa,bb) on rw_view1 to regress_view_user2;

grant insert on rw_view1 to regress_view_user2;

grant update (bb) on rw_view1 to regress_view_user2;

set session authorization regress_view_user2;

insert into rw_view1 values ('yyy',2.0,1)
  on conflict (aa) do update set bb = excluded.cc;

insert into rw_view1 values ('yyy',2.0,1)
  on conflict (aa) do update set bb = rw_view1.cc;

insert into rw_view1 values ('yyy',2.0,1)
  on conflict (aa) do update set bb = excluded.bb;

insert into rw_view1 values ('zzz',2.0,1)
  on conflict (aa) do update set bb = rw_view1.bb||'xxx';

insert into rw_view1 values ('zzz',2.0,1)
  on conflict (aa) do update set cc = 3.0;

reset session authorization;

select * from base_tbl;

set session authorization regress_view_user1;

grant select (a,b) on base_tbl to regress_view_user2;

grant insert (a,b) on base_tbl to regress_view_user2;

grant update (a,b) on base_tbl to regress_view_user2;

set session authorization regress_view_user2;

create view rw_view2 as select b as bb, c as cc, a as aa from base_tbl;

insert into rw_view2 (aa,bb) values (1,'xxx')
  on conflict (aa) do update set bb = excluded.bb;

create view rw_view3 as select b as bb, a as aa from base_tbl;

insert into rw_view3 (aa,bb) values (1,'xxx')
  on conflict (aa) do update set bb = excluded.bb;

reset session authorization;

select * from base_tbl;

set session authorization regress_view_user2;

create view rw_view4 as select aa, bb, cc FROM rw_view1;

insert into rw_view4 (aa,bb) values (1,'yyy')
  on conflict (aa) do update set bb = excluded.bb;

create view rw_view5 as select aa, bb FROM rw_view1;

insert into rw_view5 (aa,bb) values (1,'yyy')
  on conflict (aa) do update set bb = excluded.bb;

reset session authorization;

select * from base_tbl;

drop view rw_view5;

drop view rw_view4;

drop view rw_view3;

drop view rw_view2;

drop view rw_view1;

drop table base_tbl;

drop user regress_view_user1;

drop user regress_view_user2;

create table base_tab_def (a int, b text default 'Table default',
                           c text default 'Table default', d text, e text);

create view base_tab_def_view as select * from base_tab_def;

alter view base_tab_def_view alter b set default 'View default';

alter view base_tab_def_view alter d set default 'View default';

insert into base_tab_def values (1);

insert into base_tab_def values (2), (3);

insert into base_tab_def values (4, default, default, default, default);

insert into base_tab_def values (5, default, default, default, default),
                                (6, default, default, default, default);

insert into base_tab_def_view values (11);

insert into base_tab_def_view values (12), (13);

insert into base_tab_def_view values (14, default, default, default, default);

insert into base_tab_def_view values (15, default, default, default, default),
                                     (16, default, default, default, default);

insert into base_tab_def_view values (17), (default);

select * from base_tab_def order by a;

create function base_tab_def_view_instrig_func() returns trigger
as
$$
begin
  insert into base_tab_def values (new.a, new.b, new.c, new.d, new.e);
  return new;
end;
$$
language plpgsql;

create trigger base_tab_def_view_instrig instead of insert on base_tab_def_view
  for each row execute function base_tab_def_view_instrig_func();

truncate base_tab_def;

insert into base_tab_def values (1);

insert into base_tab_def values (2), (3);

insert into base_tab_def values (4, default, default, default, default);

insert into base_tab_def values (5, default, default, default, default),
                                (6, default, default, default, default);

insert into base_tab_def_view values (11);

insert into base_tab_def_view values (12), (13);

insert into base_tab_def_view values (14, default, default, default, default);

insert into base_tab_def_view values (15, default, default, default, default),
                                     (16, default, default, default, default);

insert into base_tab_def_view values (17), (default);

select * from base_tab_def order by a;

drop trigger base_tab_def_view_instrig on base_tab_def_view;

drop function base_tab_def_view_instrig_func;

create rule base_tab_def_view_ins_rule as on insert to base_tab_def_view
  do instead insert into base_tab_def values (new.a, new.b, new.c, new.d, new.e);

truncate base_tab_def;

insert into base_tab_def values (1);

insert into base_tab_def values (2), (3);

insert into base_tab_def values (4, default, default, default, default);

insert into base_tab_def values (5, default, default, default, default),
                                (6, default, default, default, default);

insert into base_tab_def_view values (11);

insert into base_tab_def_view values (12), (13);

insert into base_tab_def_view values (14, default, default, default, default);

insert into base_tab_def_view values (15, default, default, default, default),
                                     (16, default, default, default, default);

insert into base_tab_def_view values (17), (default);

select * from base_tab_def order by a;

drop rule base_tab_def_view_ins_rule on base_tab_def_view;

create rule base_tab_def_view_ins_rule as on insert to base_tab_def_view
  do also insert into base_tab_def values (new.a, new.b, new.c, new.d, new.e);

truncate base_tab_def;

insert into base_tab_def values (1);

insert into base_tab_def values (2), (3);

insert into base_tab_def values (4, default, default, default, default);

insert into base_tab_def values (5, default, default, default, default),
                                (6, default, default, default, default);

insert into base_tab_def_view values (11);

insert into base_tab_def_view values (12), (13);

insert into base_tab_def_view values (14, default, default, default, default);

insert into base_tab_def_view values (15, default, default, default, default),
                                     (16, default, default, default, default);

insert into base_tab_def_view values (17), (default);

select * from base_tab_def order by a, c NULLS LAST;

drop rule base_tab_def_view_ins_rule on base_tab_def_view;

select new.a, new.b, 'xxx';

truncate base_tab_def;

insert into base_tab_def_view values (1, default, default, default, default);

insert into base_tab_def_view values (2, default, default, default, default),
                                     (3, default, default, default, default);

select * from base_tab_def order by a, e nulls first;

drop view base_tab_def_view;

drop table base_tab_def;

create table base_tab (a serial, b int[], c text, d text default 'Table default');

create view base_tab_view as select c, a, b from base_tab;

alter view base_tab_view alter column c set default 'View default';

insert into base_tab_view (b[1], b[2], c, b[5], b[4], a, b[3])
values (1, 2, default, 5, 4, default, 3), (10, 11, 'C value', 14, 13, 100, 12);

select * from base_tab order by a;

drop view base_tab_view;

drop table base_tab;
