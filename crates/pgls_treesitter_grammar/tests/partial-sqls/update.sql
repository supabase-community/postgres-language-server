UPDATE update_test SET a = DEFAULT, b = DEFAULT;

UPDATE update_test AS t SET b = 10 WHERE t.a = 10;

UPDATE update_test t SET b = t.b + 10 WHERE t.a = 10;

UPDATE update_test t SET t.b = t.b + 10 WHERE t.a = 10;

UPDATE update_test SET a=v.i FROM (VALUES(100, 20)) AS v(i, j)
  WHERE update_test.b = v.j;

UPDATE update_test SET a = v.* FROM (VALUES(100, 20)) AS v(i, j)
  WHERE update_test.b = v.j;

UPDATE update_test SET (c,b,a) = ('bugle', b+11, DEFAULT) WHERE c = 'foo';

UPDATE update_test SET (c,b) = ('car', a+b), a = a + 1 WHERE a = 10;

UPDATE update_test SET (c,b) = ('car', a+b), b = a + 1 WHERE a = 10;

UPDATE update_test
  SET (b,a) = (select a,b from update_test where b = 41 and c = 'car')
  WHERE a = 100 AND b = 20;

UPDATE update_test o
  SET (b,a) = (select a+1,b from update_test i
               where i.a=o.a and i.b=o.b and i.c is not distinct from o.c);

UPDATE update_test SET (b,a) = (select a+1,b from update_test);

UPDATE update_test SET (b,a) = (select a+1,b from update_test where a = 1000)
  WHERE a = 11;

UPDATE update_test SET (a,b) = ROW(v.*) FROM (VALUES(21, 100)) AS v(i, j)
  WHERE update_test.a = v.i;

UPDATE update_test SET (a,b) = (v.*) FROM (VALUES(21, 101)) AS v(i, j)
  WHERE update_test.a = v.i;

UPDATE update_test AS t SET b = update_test.b + 10 WHERE t.a = 10;

UPDATE update_test SET c = repeat('x', 10000) WHERE c = 'car';

UPDATE update_test t
  SET (a, b) = (SELECT b, a FROM update_test s WHERE s.a = t.a)
  WHERE CURRENT_USER = SESSION_USER;

UPDATE part_b_10_b_20 set b = b - 6;

UPDATE part_c_100_200 set c = c - 20, d = c WHERE c = 105;

UPDATE part_b_10_b_20 set a = 'a';

UPDATE range_parted set d = d - 10 WHERE d > 10;

UPDATE range_parted set e = d;

UPDATE part_c_1_100 set c = c + 20 WHERE c = 98;

UPDATE part_b_10_b_20 set c = c + 20 returning c, b, a;

UPDATE part_b_10_b_20 set b = b - 6 WHERE c > 116 returning *;

UPDATE range_parted set b = b - 6 WHERE c > 116 returning a, b + c;

UPDATE upview set c = 199 WHERE b = 4;

UPDATE upview set c = 120 WHERE b = 4;

UPDATE upview set a = 'b', b = 15, c = 120 WHERE b = 4;

UPDATE upview set a = 'b', b = 15 WHERE b = 4;

UPDATE range_parted set c = 95 WHERE a = 'b' and b > 10 and c > 100 returning (range_parted), *;

UPDATE range_parted set c = c + 50 WHERE a = 'b' and b > 10 and c >= 96;

UPDATE range_parted set c = c + 50 WHERE a = 'b' and b > 10 and c >= 96;

UPDATE range_parted set b = 15 WHERE b = 1;

UPDATE range_parted set a = 'b', c = 151 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 151 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 150 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 122 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 120 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 112 WHERE a = 'a' and c = 200;

UPDATE range_parted set a = 'b', c = 116 WHERE a = 'a' and c = 200;

UPDATE range_parted set c = c - 50 WHERE c > 97;

update part_def set a = 'd' where a = 'c';

update part_def set a = 'a' where a = 'd';

UPDATE part_a_10_a_20 set a = 'ad' WHERE a = 'a';

UPDATE range_parted set a = 'ad' WHERE a = 'a';

UPDATE range_parted set a = 'bd' WHERE a = 'b';

UPDATE range_parted set a = 'a' WHERE a = 'ad';

UPDATE range_parted set a = 'b' WHERE a = 'bd';

UPDATE list_default set a = 'a' WHERE a = 'd';

UPDATE list_default set a = 'x' WHERE a = 'd';

update utrtest set b = b || b from (values (1), (2)) s(x) where a = s.x
  returning *, tableoid::regclass, xmin = pg_current_xact_id()::xid as xmin_ok;

update utrtest set a = 3 - a from (values (1), (2)) s(x) where a = s.x
  returning *, tableoid::regclass, xmin = pg_current_xact_id()::xid as xmin_ok;

update utrtest set a = 3 - a from (values (1), (2)) s(x) where a = s.x
  returning *, tableoid::regclass;

UPDATE sub_parted set a = 2 WHERE c = 10;

UPDATE list_parted set b = c + a WHERE a = 2;

UPDATE list_parted set c = 70 WHERE b  = 1;

UPDATE list_parted set b = 1 WHERE c = 70;

UPDATE list_parted set b = 1 WHERE c = 70;

UPDATE list_parted t1 set a = 2 FROM non_parted t2 WHERE t1.a = t2.id and a = 1;

update hpart1 set a = 3, b=4 where a = 1;

update hash_parted set b = b - 1 where b = 1;

update hash_parted set b = b + 8 where b = 1;
