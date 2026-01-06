CREATE TABLE BOX_TBL (f1 box);

INSERT INTO BOX_TBL (f1) VALUES ('(2.0,2.0,0.0,0.0)');

INSERT INTO BOX_TBL (f1) VALUES ('(1.0,1.0,3.0,3.0)');

INSERT INTO BOX_TBL (f1) VALUES ('((-8, 2), (-2, -10))');

INSERT INTO BOX_TBL (f1) VALUES ('(2.5, 2.5, 2.5,3.5)');

INSERT INTO BOX_TBL (f1) VALUES ('(3.0, 3.0,3.0,3.0)');

INSERT INTO BOX_TBL (f1) VALUES ('(2.3, 4.5)');

INSERT INTO BOX_TBL (f1) VALUES ('[1, 2, 3, 4)');

INSERT INTO BOX_TBL (f1) VALUES ('(1, 2, 3, 4]');

INSERT INTO BOX_TBL (f1) VALUES ('(1, 2, 3, 4) x');

INSERT INTO BOX_TBL (f1) VALUES ('asdfasdf(ad');

SELECT * FROM BOX_TBL;

SELECT b.*, area(b.f1) as barea
   FROM BOX_TBL b;

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 && box '(2.5,2.5,1.0,1.0)';

SELECT b1.*
   FROM BOX_TBL b1
   WHERE b1.f1 &< box '(2.0,2.0,2.5,2.5)';

SELECT b1.*
   FROM BOX_TBL b1
   WHERE b1.f1 &> box '(2.0,2.0,2.5,2.5)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 << box '(3.0,3.0,5.0,5.0)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 <= box '(3.0,3.0,5.0,5.0)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 < box '(3.0,3.0,5.0,5.0)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 = box '(3.0,3.0,5.0,5.0)';

SELECT b.f1
   FROM BOX_TBL b				-- zero area
   WHERE b.f1 > box '(3.5,3.0,4.5,3.0)';

SELECT b.f1
   FROM BOX_TBL b				-- zero area
   WHERE b.f1 >= box '(3.5,3.0,4.5,3.0)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE box '(3.0,3.0,5.0,5.0)' >> b.f1;

SELECT b.f1
   FROM BOX_TBL b
   WHERE b.f1 <@ box '(0,0,3,3)';

SELECT b.f1
   FROM BOX_TBL b
   WHERE box '(0,0,3,3)' @> b.f1;

SELECT b.f1
   FROM BOX_TBL b
   WHERE box '(1,1,3,3)' ~= b.f1;

SELECT @@(b1.f1) AS p
   FROM BOX_TBL b1;

SELECT b1.*, b2.*
   FROM BOX_TBL b1, BOX_TBL b2
   WHERE b1.f1 @> b2.f1 and not b1.f1 ~= b2.f1;

SELECT height(f1), width(f1) FROM BOX_TBL;

CREATE TEMPORARY TABLE box_temp (f1 box);

INSERT INTO box_temp
	SELECT box(point(i, i), point(i * 2, i * 2))
	FROM generate_series(1, 50) AS i;

CREATE INDEX box_spgist ON box_temp USING spgist (f1);

INSERT INTO box_temp
	VALUES (NULL),
		   ('(0,0)(0,100)'),
		   ('(-3,4.3333333333)(40,1)'),
		   ('(0,100)(0,infinity)'),
		   ('(-infinity,0)(0,infinity)'),
		   ('(-infinity,-infinity)(infinity,infinity)');

SET enable_seqscan = false;

SELECT * FROM box_temp WHERE f1 << '(10,20),(30,40)';

SELECT * FROM box_temp WHERE f1 << '(10,20),(30,40)';

SELECT * FROM box_temp WHERE f1 &< '(10,4.333334),(5,100)';

SELECT * FROM box_temp WHERE f1 &< '(10,4.333334),(5,100)';

SELECT * FROM box_temp WHERE f1 && '(15,20),(25,30)';

SELECT * FROM box_temp WHERE f1 && '(15,20),(25,30)';

SELECT * FROM box_temp WHERE f1 &> '(40,30),(45,50)';

SELECT * FROM box_temp WHERE f1 &> '(40,30),(45,50)';

SELECT * FROM box_temp WHERE f1 >> '(30,40),(40,30)';

SELECT * FROM box_temp WHERE f1 >> '(30,40),(40,30)';

SELECT * FROM box_temp WHERE f1 <<| '(10,4.33334),(5,100)';

SELECT * FROM box_temp WHERE f1 <<| '(10,4.33334),(5,100)';

SELECT * FROM box_temp WHERE f1 &<| '(10,4.3333334),(5,1)';

SELECT * FROM box_temp WHERE f1 &<| '(10,4.3333334),(5,1)';

SELECT * FROM box_temp WHERE f1 |&> '(49.99,49.99),(49.99,49.99)';

SELECT * FROM box_temp WHERE f1 |&> '(49.99,49.99),(49.99,49.99)';

SELECT * FROM box_temp WHERE f1 |>> '(37,38),(39,40)';

SELECT * FROM box_temp WHERE f1 |>> '(37,38),(39,40)';

SELECT * FROM box_temp WHERE f1 @> '(10,11),(15,16)';

SELECT * FROM box_temp WHERE f1 @> '(10,11),(15,15)';

SELECT * FROM box_temp WHERE f1 <@ '(10,15),(30,35)';

SELECT * FROM box_temp WHERE f1 <@ '(10,15),(30,35)';

SELECT * FROM box_temp WHERE f1 ~= '(20,20),(40,40)';

SELECT * FROM box_temp WHERE f1 ~= '(20,20),(40,40)';

RESET enable_seqscan;

DROP INDEX box_spgist;

CREATE TABLE quad_box_tbl (id int, b box);

INSERT INTO quad_box_tbl
  SELECT (x - 1) * 100 + y, box(point(x * 10, y * 10), point(x * 10 + 5, y * 10 + 5))
  FROM generate_series(1, 100) x,
       generate_series(1, 100) y;

INSERT INTO quad_box_tbl
  SELECT i, '((200, 300),(210, 310))'
  FROM generate_series(10001, 11000) AS i;

INSERT INTO quad_box_tbl
VALUES
  (11001, NULL),
  (11002, NULL),
  (11003, '((-infinity,-infinity),(infinity,infinity))'),
  (11004, '((-infinity,100),(-infinity,500))'),
  (11005, '((-infinity,-infinity),(700,infinity))');

CREATE INDEX quad_box_tbl_idx ON quad_box_tbl USING spgist(b);

SET enable_seqscan = ON;

SET enable_indexscan = OFF;

SET enable_bitmapscan = OFF;

CREATE TABLE quad_box_tbl_ord_seq1 AS
SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl;

CREATE TABLE quad_box_tbl_ord_seq2 AS
SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl WHERE b <@ box '((200,300),(500,600))';

SET enable_seqscan = OFF;

SET enable_indexscan = ON;

SET enable_bitmapscan = ON;

SELECT count(*) FROM quad_box_tbl WHERE b <<  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b &<  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b &&  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b &>  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b >>  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b >>  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b <<| box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b &<| box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b |&> box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b |>> box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b @>  box '((201,301),(202,303))';

SELECT count(*) FROM quad_box_tbl WHERE b <@  box '((100,200),(300,500))';

SELECT count(*) FROM quad_box_tbl WHERE b ~=  box '((200,300),(205,305))';

SET enable_indexscan = ON;

SET enable_bitmapscan = OFF;

SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl;

CREATE TEMP TABLE quad_box_tbl_ord_idx1 AS
SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl;

SELECT *
FROM quad_box_tbl_ord_seq1 seq FULL JOIN quad_box_tbl_ord_idx1 idx
	ON seq.n = idx.n AND seq.id = idx.id AND
		(seq.dist = idx.dist OR seq.dist IS NULL AND idx.dist IS NULL)
WHERE seq.id IS NULL OR idx.id IS NULL;

SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl WHERE b <@ box '((200,300),(500,600))';

CREATE TEMP TABLE quad_box_tbl_ord_idx2 AS
SELECT rank() OVER (ORDER BY b <-> point '123,456') n, b <-> point '123,456' dist, id
FROM quad_box_tbl WHERE b <@ box '((200,300),(500,600))';

SELECT *
FROM quad_box_tbl_ord_seq2 seq FULL JOIN quad_box_tbl_ord_idx2 idx
	ON seq.n = idx.n AND seq.id = idx.id AND
		(seq.dist = idx.dist OR seq.dist IS NULL AND idx.dist IS NULL)
WHERE seq.id IS NULL OR idx.id IS NULL;

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

SELECT pg_input_is_valid('200', 'box');

SELECT * FROM pg_input_error_info('200', 'box');

SELECT pg_input_is_valid('((200,300),(500, xyz))', 'box');

SELECT * FROM pg_input_error_info('((200,300),(500, xyz))', 'box');
