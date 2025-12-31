SET extra_float_digits = 0;

INSERT INTO POINT_TBL(f1) VALUES ('asdfasdf');

INSERT INTO POINT_TBL(f1) VALUES ('(10.0 10.0)');

INSERT INTO POINT_TBL(f1) VALUES ('(10.0, 10.0) x');

INSERT INTO POINT_TBL(f1) VALUES ('(10.0,10.0');

INSERT INTO POINT_TBL(f1) VALUES ('(10.0, 1e+500)');

SELECT * FROM POINT_TBL;

SELECT p.* FROM POINT_TBL p WHERE p.f1 << '(0.0, 0.0)';

SELECT p.* FROM POINT_TBL p WHERE '(0.0,0.0)' >> p.f1;

SELECT p.* FROM POINT_TBL p WHERE '(0.0,0.0)' |>> p.f1;

SELECT p.* FROM POINT_TBL p WHERE p.f1 <<| '(0.0, 0.0)';

SELECT p.* FROM POINT_TBL p WHERE p.f1 ~= '(5.1, 34.5)';

SELECT p.* FROM POINT_TBL p
   WHERE p.f1 <@ box '(0,0,100,100)';

SELECT p.* FROM POINT_TBL p
   WHERE box '(0,0,100,100)' @> p.f1;

SELECT p.* FROM POINT_TBL p
   WHERE not p.f1 <@ box '(0,0,100,100)';

SELECT p.* FROM POINT_TBL p
   WHERE p.f1 <@ path '[(0,0),(-10,0),(-10,10)]';

SELECT p.* FROM POINT_TBL p
   WHERE not box '(0,0,100,100)' @> p.f1;

SELECT p.f1, p.f1 <-> point '(0,0)' AS dist
   FROM POINT_TBL p
   ORDER BY dist;

SELECT p1.f1 AS point1, p2.f1 AS point2, p1.f1 <-> p2.f1 AS dist
   FROM POINT_TBL p1, POINT_TBL p2
   ORDER BY dist, p1.f1[0], p2.f1[0];

SELECT p1.f1 AS point1, p2.f1 AS point2
   FROM POINT_TBL p1, POINT_TBL p2
   WHERE (p1.f1 <-> p2.f1) > 3;

SELECT p1.f1 AS point1, p2.f1 AS point2, (p1.f1 <-> p2.f1) AS distance
   FROM POINT_TBL p1, POINT_TBL p2
   WHERE (p1.f1 <-> p2.f1) > 3 and p1.f1 << p2.f1
   ORDER BY distance, p1.f1[0], p2.f1[0];

SELECT p1.f1 AS point1, p2.f1 AS point2, (p1.f1 <-> p2.f1) AS distance
   FROM POINT_TBL p1, POINT_TBL p2
   WHERE (p1.f1 <-> p2.f1) > 3 and p1.f1 << p2.f1 and p1.f1 |>> p2.f1
   ORDER BY distance;

CREATE TEMP TABLE point_gist_tbl(f1 point);

INSERT INTO point_gist_tbl SELECT '(0,0)' FROM generate_series(0,1000);

CREATE INDEX point_gist_tbl_index ON point_gist_tbl USING gist (f1);

INSERT INTO point_gist_tbl VALUES ('(0.0000009,0.0000009)');

SET enable_seqscan TO true;

SET enable_indexscan TO false;

SET enable_bitmapscan TO false;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 ~= '(0.0000009,0.0000009)'::point;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 <@ '(0.0000009,0.0000009),(0.0000009,0.0000009)'::box;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 ~= '(0.0000018,0.0000018)'::point;

SET enable_seqscan TO false;

SET enable_indexscan TO true;

SET enable_bitmapscan TO true;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 ~= '(0.0000009,0.0000009)'::point;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 <@ '(0.0000009,0.0000009),(0.0000009,0.0000009)'::box;

SELECT COUNT(*) FROM point_gist_tbl WHERE f1 ~= '(0.0000018,0.0000018)'::point;

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

SELECT pg_input_is_valid('1,y', 'point');

SELECT * FROM pg_input_error_info('1,y', 'point');
