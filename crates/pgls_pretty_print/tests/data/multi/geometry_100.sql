SET extra_float_digits TO -3;

SELECT center(f1) AS center
   FROM BOX_TBL;

SELECT (@@ f1) AS center
   FROM BOX_TBL;

SELECT point(f1) AS center
   FROM CIRCLE_TBL;

SELECT (@@ f1) AS center
   FROM CIRCLE_TBL;

SELECT (@@ f1) AS center
   FROM POLYGON_TBL
   WHERE (# f1) > 2;

SELECT p1.f1
   FROM POINT_TBL p1
   WHERE ishorizontal(p1.f1, point '(0,0)');

SELECT p1.f1
   FROM POINT_TBL p1
   WHERE p1.f1 ?- point '(0,0)';

SELECT p1.f1
   FROM POINT_TBL p1
   WHERE isvertical(p1.f1, point '(5.1,34.5)');

SELECT p1.f1
   FROM POINT_TBL p1
   WHERE p1.f1 ?| point '(5.1,34.5)';

SELECT p1.f1, p2.f1, slope(p1.f1, p2.f1) FROM POINT_TBL p1, POINT_TBL p2;

SELECT p1.f1, p2.f1, p1.f1 + p2.f1 FROM POINT_TBL p1, POINT_TBL p2;

SELECT p1.f1, p2.f1, p1.f1 - p2.f1 FROM POINT_TBL p1, POINT_TBL p2;

SELECT p1.f1, p2.f1, p1.f1 * p2.f1 FROM POINT_TBL p1, POINT_TBL p2 WHERE p1.f1[0] BETWEEN 1 AND 1000;

SELECT p1.f1, p2.f1, p1.f1 * p2.f1 FROM POINT_TBL p1, POINT_TBL p2 WHERE p1.f1[0] < 1;

SELECT p1.f1, p2.f1, p1.f1 / p2.f1 FROM POINT_TBL p1, POINT_TBL p2 WHERE p2.f1[0] BETWEEN 1 AND 1000;

SELECT p1.f1, p2.f1, p1.f1 / p2.f1 FROM POINT_TBL p1, POINT_TBL p2 WHERE p2.f1[0] > 1000;

SELECT p1.f1, p2.f1, p1.f1 / p2.f1 FROM POINT_TBL p1, POINT_TBL p2 WHERE p2.f1 ~= '(0,0)'::point;

SELECT p.f1, l.s, p.f1 <-> l.s AS dist_pl, l.s <-> p.f1 AS dist_lp FROM POINT_TBL p, LINE_TBL l;

SELECT p.f1, l.s, p.f1 <-> l.s AS dist_ps, l.s <-> p.f1 AS dist_sp FROM POINT_TBL p, LSEG_TBL l;

SELECT p.f1, b.f1, p.f1 <-> b.f1 AS dist_pb, b.f1 <-> p.f1 AS dist_bp FROM POINT_TBL p, BOX_TBL b;

SELECT p.f1, p1.f1, p.f1 <-> p1.f1 AS dist_ppath, p1.f1 <-> p.f1 AS dist_pathp FROM POINT_TBL p, PATH_TBL p1;

SELECT p.f1, p1.f1, p.f1 <-> p1.f1 AS dist_ppoly, p1.f1 <-> p.f1 AS dist_polyp FROM POINT_TBL p, POLYGON_TBL p1;

SELECT p1.f1, p2.f1, line(p1.f1, p2.f1)
  FROM POINT_TBL p1, POINT_TBL p2 WHERE p1.f1 <> p2.f1;

SELECT p.f1, l.s, p.f1 ## l.s FROM POINT_TBL p, LINE_TBL l;

SELECT p.f1, l.s, p.f1 ## l.s FROM POINT_TBL p, LSEG_TBL l;

SELECT p.f1, b.f1, p.f1 ## b.f1 FROM POINT_TBL p, BOX_TBL b;

SELECT p.f1, l.s FROM POINT_TBL p, LINE_TBL l WHERE p.f1 <@ l.s;

SELECT p.f1, l.s FROM POINT_TBL p, LSEG_TBL l WHERE p.f1 <@ l.s;

SELECT p.f1, p1.f1 FROM POINT_TBL p, PATH_TBL p1 WHERE p.f1 <@ p1.f1;

SELECT s FROM LINE_TBL WHERE ?| s;

SELECT s FROM LINE_TBL WHERE ?- s;

SELECT l1.s, l2.s FROM LINE_TBL l1, LINE_TBL l2 WHERE l1.s = l2.s;

SELECT l1.s, l2.s FROM LINE_TBL l1, LINE_TBL l2 WHERE l1.s ?|| l2.s;

SELECT l1.s, l2.s FROM LINE_TBL l1, LINE_TBL l2 WHERE l1.s ?-| l2.s;

SELECT l1.s, l2.s, l1.s <-> l2.s FROM LINE_TBL l1, LINE_TBL l2;

SELECT l1.s, l2.s FROM LINE_TBL l1, LINE_TBL l2 WHERE l1.s ?# l2.s;

SELECT l.s, b.f1 FROM LINE_TBL l, BOX_TBL b WHERE l.s ?# b.f1;

SELECT l1.s, l2.s, l1.s # l2.s FROM LINE_TBL l1, LINE_TBL l2;

SELECT l.s, l1.s, l.s ## l1.s FROM LINE_TBL l, LSEG_TBL l1;

SELECT p.f1, l.s, l.s # p.f1 AS intersection
   FROM LSEG_TBL l, POINT_TBL p;

SELECT s, @-@ s FROM LSEG_TBL;

SELECT s FROM LSEG_TBL WHERE ?| s;

SELECT s FROM LSEG_TBL WHERE ?- s;

SELECT s, @@ s FROM LSEG_TBL;

SELECT s, s::point FROM LSEG_TBL;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s < l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s <= l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s = l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s >= l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s > l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s != l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s ?|| l2.s;

SELECT l1.s, l2.s FROM LSEG_TBL l1, LSEG_TBL l2 WHERE l1.s ?-| l2.s;

SELECT l.s, l1.s, l.s <-> l1.s AS dist_sl, l1.s <-> l.s AS dist_ls FROM LSEG_TBL l, LINE_TBL l1;

SELECT l1.s, l2.s, l1.s <-> l2.s FROM LSEG_TBL l1, LSEG_TBL l2;

SELECT l.s, b.f1, l.s <-> b.f1 AS dist_sb, b.f1 <-> l.s AS dist_bs FROM LSEG_TBL l, BOX_TBL b;

SELECT l.s, l1.s FROM LSEG_TBL l, LINE_TBL l1 WHERE l.s ?# l1.s;

SELECT l.s, b.f1 FROM LSEG_TBL l, BOX_TBL b WHERE l.s ?# b.f1;

SELECT l1.s, l2.s, l1.s # l2.s FROM LSEG_TBL l1, LSEG_TBL l2;

SELECT l1.s, l2.s, l1.s ## l2.s FROM LSEG_TBL l1, LSEG_TBL l2;

SELECT l.s, b.f1, l.s ## b.f1 FROM LSEG_TBL l, BOX_TBL b;

SELECT l.s, l1.s FROM LSEG_TBL l, LINE_TBL l1 WHERE l.s <@ l1.s;

SELECT l.s, b.f1 FROM LSEG_TBL l, BOX_TBL b WHERE l.s <@ b.f1;

SELECT box(f1) AS box FROM CIRCLE_TBL;

SELECT b.f1 + p.f1 AS translation
   FROM BOX_TBL b, POINT_TBL p;

SELECT b.f1 - p.f1 AS translation
   FROM BOX_TBL b, POINT_TBL p;

SELECT b.f1, p.f1, b.f1 * p.f1 FROM BOX_TBL b, POINT_TBL p WHERE p.f1[0] BETWEEN 1 AND 1000;

SELECT b.f1, p.f1, b.f1 * p.f1 FROM BOX_TBL b, POINT_TBL p WHERE p.f1[0] > 1000;

SELECT b.f1, p.f1, b.f1 / p.f1 FROM BOX_TBL b, POINT_TBL p WHERE p.f1[0] BETWEEN 1 AND 1000;

SELECT f1::box
	FROM POINT_TBL;

SELECT bound_box(a.f1, b.f1)
	FROM BOX_TBL a, BOX_TBL b;

SELECT b1.f1, b2.f1, b1.f1 <^ b2.f1 FROM BOX_TBL b1, BOX_TBL b2;

SELECT b1.f1, b2.f1, b1.f1 >^ b2.f1 FROM BOX_TBL b1, BOX_TBL b2;

SELECT b1.f1, b2.f1, b1.f1 # b2.f1 FROM BOX_TBL b1, BOX_TBL b2;

SELECT f1, diagonal(f1) FROM BOX_TBL;

SELECT b1.f1, b2.f1, b1.f1 <-> b2.f1 FROM BOX_TBL b1, BOX_TBL b2;

SELECT f1, npoints(f1) FROM PATH_TBL;

SELECT f1, area(f1) FROM PATH_TBL;

SELECT f1, @-@ f1 FROM PATH_TBL;

SELECT f1, f1::polygon FROM PATH_TBL WHERE isclosed(f1);

SELECT f1, f1::polygon FROM PATH_TBL WHERE isopen(f1);

SELECT p1.f1, p2.f1 FROM PATH_TBL p1, PATH_TBL p2 WHERE p1.f1 < p2.f1;

SELECT p1.f1, p2.f1 FROM PATH_TBL p1, PATH_TBL p2 WHERE p1.f1 <= p2.f1;

SELECT p1.f1, p2.f1 FROM PATH_TBL p1, PATH_TBL p2 WHERE p1.f1 = p2.f1;

SELECT p1.f1, p2.f1 FROM PATH_TBL p1, PATH_TBL p2 WHERE p1.f1 >= p2.f1;

SELECT p1.f1, p2.f1 FROM PATH_TBL p1, PATH_TBL p2 WHERE p1.f1 > p2.f1;

SELECT p1.f1, p2.f1, p1.f1 + p2.f1 FROM PATH_TBL p1, PATH_TBL p2;

SELECT p.f1, p1.f1, p.f1 + p1.f1 FROM PATH_TBL p, POINT_TBL p1;

SELECT p.f1, p1.f1, p.f1 - p1.f1 FROM PATH_TBL p, POINT_TBL p1;

SELECT p.f1, p1.f1, p.f1 * p1.f1 FROM PATH_TBL p, POINT_TBL p1;

SELECT p.f1, p1.f1, p.f1 / p1.f1 FROM PATH_TBL p, POINT_TBL p1 WHERE p1.f1[0] BETWEEN 1 AND 1000;

SELECT p.f1, p1.f1, p.f1 / p1.f1 FROM PATH_TBL p, POINT_TBL p1 WHERE p1.f1 ~= '(0,0)'::point;

SELECT p1.f1, p2.f1, p1.f1 <-> p2.f1 FROM PATH_TBL p1, PATH_TBL p2;

SELECT p.f1, poly.f1, poly.f1 @> p.f1 AS contains
   FROM POLYGON_TBL poly, POINT_TBL p;

SELECT p.f1, poly.f1, p.f1 <@ poly.f1 AS contained
   FROM POLYGON_TBL poly, POINT_TBL p;

SELECT npoints(f1) AS npoints, f1 AS polygon
   FROM POLYGON_TBL;

SELECT polygon(f1)
   FROM BOX_TBL;

SELECT polygon(f1)
   FROM PATH_TBL WHERE isclosed(f1);

SELECT f1 AS open_path, polygon( pclose(f1)) AS polygon
   FROM PATH_TBL
   WHERE isopen(f1);

SELECT f1, f1::box FROM POLYGON_TBL;

SELECT f1, f1::path FROM POLYGON_TBL;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 ~= p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 <@ p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 @> p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 && p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 << p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 &< p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 >> p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 &> p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 <<| p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 &<| p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 |>> p2.f1;

SELECT p1.f1, p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2 WHERE p1.f1 |&> p2.f1;

SELECT p1.f1, p2.f1, p1.f1 <-> p2.f1 FROM POLYGON_TBL p1, POLYGON_TBL p2;

SELECT circle(f1, 50.0)
   FROM POINT_TBL;

SELECT circle(f1)
   FROM BOX_TBL;

SELECT circle(f1)
   FROM POLYGON_TBL
   WHERE (# f1) >= 3;

SELECT c1.f1 AS circle, p1.f1 AS point, (p1.f1 <-> c1.f1) AS distance
   FROM CIRCLE_TBL c1, POINT_TBL p1
   WHERE (p1.f1 <-> c1.f1) > 0
   ORDER BY distance, area(c1.f1), p1.f1[0];

SELECT f1, f1::polygon FROM CIRCLE_TBL WHERE f1 >= '<(0,0),1>';

SELECT f1, polygon(8, f1) FROM CIRCLE_TBL WHERE f1 >= '<(0,0),1>';

SELECT f1, polygon(1, f1) FROM CIRCLE_TBL WHERE f1 >= '<(0,0),1>';

SELECT f1, polygon(10, f1) FROM CIRCLE_TBL WHERE f1 < '<(0,0),1>';

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 ~= c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 && c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 &< c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 << c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 >> c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 &> c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 <@ c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 @> c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 <<| c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 |>> c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 &<| c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 |&> c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 = c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 != c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 < c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 > c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 <= c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 >= c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 < c2.f1;

SELECT c1.f1, c2.f1 FROM CIRCLE_TBL c1, CIRCLE_TBL c2 WHERE c1.f1 < c2.f1;

SELECT c.f1, p.f1, c.f1 + p.f1 FROM CIRCLE_TBL c, POINT_TBL p;

SELECT c.f1, p.f1, c.f1 - p.f1 FROM CIRCLE_TBL c, POINT_TBL p;

SELECT c.f1, p.f1, c.f1 * p.f1 FROM CIRCLE_TBL c, POINT_TBL p;

SELECT c.f1, p.f1, c.f1 / p.f1 FROM CIRCLE_TBL c, POINT_TBL p WHERE p.f1[0] BETWEEN 1 AND 1000;

SELECT c.f1, p.f1, c.f1 / p.f1 FROM CIRCLE_TBL c, POINT_TBL p WHERE p.f1[0] > 1000;

SELECT c.f1, p.f1, c.f1 / p.f1 FROM CIRCLE_TBL c, POINT_TBL p WHERE p.f1 ~= '(0,0)'::point;

SELECT c.f1, p.f1, c.f1 <-> p.f1 FROM CIRCLE_TBL c, POLYGON_TBL p;

CREATE INDEX gcircleind ON circle_tbl USING gist (f1);

SELECT * FROM circle_tbl WHERE f1 && circle(point(1,-2), 1)
    ORDER BY area(f1);

SELECT * FROM circle_tbl WHERE f1 && circle(point(1,-2), 1)
    ORDER BY area(f1);

SELECT * FROM circle_tbl WHERE f1 && circle(point(1,-2), 1)
    ORDER BY area(f1);

CREATE INDEX gpolygonind ON polygon_tbl USING gist (f1);

SELECT * FROM polygon_tbl WHERE f1 @> '((1,1),(2,2),(2,1))'::polygon
    ORDER BY (poly_center(f1))[0];

SELECT * FROM polygon_tbl WHERE f1 @> '((1,1),(2,2),(2,1))'::polygon
    ORDER BY (poly_center(f1))[0];

SELECT * FROM polygon_tbl WHERE f1 @> '((1,1),(2,2),(2,1))'::polygon
    ORDER BY (poly_center(f1))[0];

SELECT pg_input_is_valid('(1', 'circle');

SELECT * FROM pg_input_error_info('1,', 'circle');

SELECT pg_input_is_valid('(1,2),-1', 'circle');

SELECT * FROM pg_input_error_info('(1,2),-1', 'circle');
