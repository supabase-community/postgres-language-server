CREATE TABLE mvtest_t (id int NOT NULL PRIMARY KEY, type text NOT NULL, amt numeric NOT NULL);

INSERT INTO mvtest_t VALUES
  (1, 'x', 2),
  (2, 'x', 3),
  (3, 'y', 5),
  (4, 'y', 7),
  (5, 'z', 11);

CREATE VIEW mvtest_tv AS SELECT type, sum(amt) AS totamt FROM mvtest_t GROUP BY type;

SELECT * FROM mvtest_tv ORDER BY type;

CREATE MATERIALIZED VIEW mvtest_tm AS SELECT type, sum(amt) AS totamt FROM mvtest_t GROUP BY type WITH NO DATA;

CREATE MATERIALIZED VIEW mvtest_tm AS SELECT type, sum(amt) AS totamt FROM mvtest_t GROUP BY type WITH NO DATA;

SELECT relispopulated FROM pg_class WHERE oid = 'mvtest_tm'::regclass;

SELECT * FROM mvtest_tm ORDER BY type;

REFRESH MATERIALIZED VIEW mvtest_tm;

SELECT relispopulated FROM pg_class WHERE oid = 'mvtest_tm'::regclass;

CREATE UNIQUE INDEX mvtest_tm_type ON mvtest_tm (type);

SELECT * FROM mvtest_tm ORDER BY type;

CREATE MATERIALIZED VIEW mvtest_tvm AS SELECT * FROM mvtest_tv ORDER BY type;

CREATE MATERIALIZED VIEW mvtest_tvm AS SELECT * FROM mvtest_tv ORDER BY type;

SELECT * FROM mvtest_tvm;

CREATE MATERIALIZED VIEW mvtest_tmm AS SELECT sum(totamt) AS grandtot FROM mvtest_tm;

CREATE MATERIALIZED VIEW mvtest_tvmm AS SELECT sum(totamt) AS grandtot FROM mvtest_tvm;

CREATE UNIQUE INDEX mvtest_tvmm_expr ON mvtest_tvmm ((grandtot > 0));

CREATE UNIQUE INDEX mvtest_tvmm_pred ON mvtest_tvmm (grandtot) WHERE grandtot < 0;

CREATE VIEW mvtest_tvv AS SELECT sum(totamt) AS grandtot FROM mvtest_tv;

CREATE MATERIALIZED VIEW mvtest_tvvm AS SELECT * FROM mvtest_tvv;

CREATE MATERIALIZED VIEW mvtest_tvvm AS SELECT * FROM mvtest_tvv;

CREATE VIEW mvtest_tvvmv AS SELECT * FROM mvtest_tvvm;

CREATE MATERIALIZED VIEW mvtest_bb AS SELECT * FROM mvtest_tvvmv;

CREATE INDEX mvtest_aa ON mvtest_bb (grandtot);

CREATE SCHEMA mvtest_mvschema;

ALTER MATERIALIZED VIEW mvtest_tvm SET SCHEMA mvtest_mvschema;

SET search_path = mvtest_mvschema, public;

INSERT INTO mvtest_t VALUES (6, 'z', 13);

SELECT * FROM mvtest_tm ORDER BY type;

SELECT * FROM mvtest_tvm ORDER BY type;

REFRESH MATERIALIZED VIEW CONCURRENTLY mvtest_tm;

REFRESH MATERIALIZED VIEW mvtest_tvm;

SELECT * FROM mvtest_tm ORDER BY type;

SELECT * FROM mvtest_tvm ORDER BY type;

RESET search_path;

SELECT * FROM mvtest_tmm;

SELECT * FROM mvtest_tvmm;

SELECT * FROM mvtest_tvvm;

SELECT * FROM mvtest_tmm;

SELECT * FROM mvtest_tvmm;

SELECT * FROM mvtest_tvvm;

REFRESH MATERIALIZED VIEW mvtest_tmm;

REFRESH MATERIALIZED VIEW CONCURRENTLY mvtest_tvmm;

REFRESH MATERIALIZED VIEW mvtest_tvmm;

REFRESH MATERIALIZED VIEW mvtest_tvvm;

SELECT * FROM mvtest_tmm;

SELECT * FROM mvtest_tvmm;

SELECT * FROM mvtest_tvvm;

SELECT * FROM mvtest_tmm;

SELECT * FROM mvtest_tvmm;

SELECT * FROM mvtest_tvvm;

DROP MATERIALIZED VIEW IF EXISTS no_such_mv;

REFRESH MATERIALIZED VIEW CONCURRENTLY mvtest_tvmm
