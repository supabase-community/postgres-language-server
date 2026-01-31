SELECT
  '(0,0)'::tid as tid00,
  '(0,1)'::tid as tid01,
  '(-1,0)'::tid as tidm10,
  '(4294967295,65535)'::tid as tidmax;

SELECT '(4294967296,1)'::tid;

SELECT '(1,65536)'::tid;

SELECT pg_input_is_valid('(0)', 'tid');

SELECT * FROM pg_input_error_info('(0)', 'tid');

SELECT pg_input_is_valid('(0,-1)', 'tid');

SELECT * FROM pg_input_error_info('(0,-1)', 'tid');

CREATE TABLE tid_tab (a int);

INSERT INTO tid_tab VALUES (1), (2);

SELECT min(ctid) FROM tid_tab;

SELECT max(ctid) FROM tid_tab;

TRUNCATE tid_tab;

CREATE MATERIALIZED VIEW tid_matview AS SELECT a FROM tid_tab;

SELECT currtid2('tid_matview'::text, '(0,1)'::tid);

INSERT INTO tid_tab VALUES (1);

REFRESH MATERIALIZED VIEW tid_matview;

SELECT currtid2('tid_matview'::text, '(0,1)'::tid);

DROP MATERIALIZED VIEW tid_matview;

TRUNCATE tid_tab;

CREATE SEQUENCE tid_seq;

SELECT currtid2('tid_seq'::text, '(0,1)'::tid);

DROP SEQUENCE tid_seq;

CREATE INDEX tid_ind ON tid_tab(a);

SELECT currtid2('tid_ind'::text, '(0,1)'::tid);

DROP INDEX tid_ind;

CREATE TABLE tid_part (a int) PARTITION BY RANGE (a);

SELECT currtid2('tid_part'::text, '(0,1)'::tid);

DROP TABLE tid_part;

CREATE VIEW tid_view_no_ctid AS SELECT a FROM tid_tab;

SELECT currtid2('tid_view_no_ctid'::text, '(0,1)'::tid);

DROP VIEW tid_view_no_ctid;

CREATE VIEW tid_view_with_ctid AS SELECT ctid, a FROM tid_tab;

SELECT currtid2('tid_view_with_ctid'::text, '(0,1)'::tid);

INSERT INTO tid_tab VALUES (1);

SELECT currtid2('tid_view_with_ctid'::text, '(0,1)'::tid);

DROP VIEW tid_view_with_ctid;

TRUNCATE tid_tab;

CREATE VIEW tid_view_fake_ctid AS SELECT 1 AS ctid, 2 AS a;

SELECT currtid2('tid_view_fake_ctid'::text, '(0,1)'::tid);

DROP VIEW tid_view_fake_ctid;

DROP TABLE tid_tab CASCADE;
