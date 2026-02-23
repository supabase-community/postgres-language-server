insert into inserttest (col1, col2, col3) values (DEFAULT, DEFAULT, DEFAULT);
insert into inserttest (col2, col3) values (3, DEFAULT);
insert into inserttest (col1, col2, col3) values (DEFAULT, 5, DEFAULT);
insert into inserttest values (DEFAULT, 5, 'test');
insert into inserttest values (DEFAULT, 7);
insert into inserttest (col1, col2, col3) values (DEFAULT, DEFAULT);
insert into inserttest (col1, col2, col3) values (1, 2);
insert into inserttest (col1) values (1, 2);
insert into inserttest (col1) values (DEFAULT, DEFAULT);
insert into inserttest values(10, 20, '40'), (-1, 2, DEFAULT),
    ((select 2), (select i from (values(3)) as foo (i)), 'values are fun!');
insert into inserttest values(30, 50, repeat('x', 10000));
INSERT INTO large_tuple_test (select 1, NULL);
INSERT INTO large_tuple_test (select 2, repeat('a', 1000));
INSERT INTO large_tuple_test (select 3, NULL);
INSERT INTO large_tuple_test (select 4, repeat('a', 8126));
insert into inserttest (f2[1], f2[2]) values (1,2);
insert into inserttest (f2[1], f2[2]) values (3,4), (5,6);
insert into inserttest (f2[1], f2[2]) select 7,8;
insert into inserttest (f2[1], f2[2]) values (1,default);  -- not supported
insert into inserttest (f3.if1, f3.if2) values (1,array['foo']);
insert into inserttest (f3.if1, f3.if2) values (1,'{foo}'), (2,'{bar}');
insert into inserttest (f3.if1, f3.if2) select 3, '{baz,quux}';
insert into inserttest (f3.if1, f3.if2) values (1,default);  -- not supported
insert into inserttest (f3.if2[1], f3.if2[2]) values ('foo', 'bar');
insert into inserttest (f3.if2[1], f3.if2[2]) values ('foo', 'bar'), ('baz', 'quux');
insert into inserttest (f3.if2[1], f3.if2[2]) select 'bear', 'beer';
insert into inserttest (f4[1].if2[1], f4[1].if2[2]) values ('foo', 'bar');
insert into inserttest (f4[1].if2[1], f4[1].if2[2]) values ('foo', 'bar'), ('baz', 'quux');
insert into inserttest (f4[1].if2[1], f4[1].if2[2]) select 'bear', 'beer';
insert into inserttesta (f2[1], f2[2]) values (1,2);
insert into inserttesta (f2[1], f2[2]) values (3,4), (5,6);
insert into inserttesta (f2[1], f2[2]) select 7,8;
insert into inserttesta (f2[1], f2[2]) values (1,default);  -- not supported
insert into inserttesta (f2[1], f2[2]) values (0,2);
insert into inserttesta (f2[1], f2[2]) values (3,4), (0,6);
insert into inserttesta (f2[1], f2[2]) select 0,8;
insert into inserttestb (f3.if1, f3.if2) values (1,array['foo']);
insert into inserttestb (f3.if1, f3.if2) values (1,'{foo}'), (2,'{bar}');
insert into inserttestb (f3.if1, f3.if2) select 3, '{baz,quux}';
insert into inserttestb (f3.if1, f3.if2) values (1,default);  -- not supported
insert into inserttestb (f3.if1, f3.if2) values (1,array[null]);
insert into inserttestb (f3.if1, f3.if2) values (1,'{null}'), (2,'{bar}');
insert into inserttestb (f3.if1, f3.if2) select 3, '{null,quux}';
insert into inserttestb (f3.if2[1], f3.if2[2]) values ('foo', 'bar');
insert into inserttestb (f3.if2[1], f3.if2[2]) values ('foo', 'bar'), ('baz', 'quux');
insert into inserttestb (f3.if2[1], f3.if2[2]) select 'bear', 'beer';
insert into inserttestb (f3, f4[1].if2[1], f4[1].if2[2]) values (row(1,'{x}'), 'foo', 'bar');
insert into inserttestb (f3, f4[1].if2[1], f4[1].if2[2]) values (row(1,'{x}'), 'foo', 'bar'), (row(2,'{y}'), 'baz', 'quux');
insert into inserttestb (f3, f4[1].if2[1], f4[1].if2[2]) select row(1,'{x}')::insert_test_domain, 'bear', 'beer';
insert into inserttesta (f1[1]) values (1);  -- fail
insert into inserttesta (f1[1], f1[2]) values (1, 2);
insert into inserttestb (f1.if1) values (1);  -- fail
insert into inserttestb (f1.if1, f1.if2) values (1, '{foo}');
insert into range_parted values ('a', 11);
insert into part1 values ('a', 11);
insert into part1 values ('b', 1);
insert into part1 values ('a', 1);
insert into part4 values ('b', 21);
insert into part4 values ('a', 10);
insert into part4 values ('b', 10);
insert into part1 values (null);
insert into part1 values (1);
insert into part_aa_bb values ('cc', 1);
insert into part_aa_bb values ('AAa', 1);
insert into part_aa_bb values (null);
insert into part_cc_dd values ('cC', 1);
insert into part_null values (null, 0);
insert into part_default values ('aa', 2);
insert into part_default values (null, 2);
insert into part_default values ('Zz', 2);
insert into part_ee_ff1 values ('EE', 11);
insert into part_default_p2 values ('gg', 43);
insert into part_ee_ff1 values ('cc', 1);
insert into part_default values ('gg', 43);
insert into part_ee_ff1 values ('ff', 1);
insert into part_ee_ff2 values ('ff', 11);
insert into part_default_p1 values ('cd', 25);
insert into part_default_p2 values ('de', 35);
insert into list_parted values ('ab', 21);
insert into list_parted values ('xx', 1);
insert into list_parted values ('yy', 2);
insert into range_parted values ('a', 0);
insert into range_parted values ('a', 1);
insert into range_parted values ('a', 10);
insert into range_parted values ('a', 20);
insert into range_parted values ('b', 1);
insert into range_parted values ('b', 10);
insert into range_parted values ('a');
insert into part_def values ('b', 10);
insert into part_def values ('c', 10);
insert into range_parted values (null, null);
insert into range_parted values ('a', null);
insert into range_parted values (null, 19);
insert into range_parted values ('b', 20);
insert into list_parted values (null, 1);
insert into list_parted (a) values ('aA');
insert into list_parted values ('EE', 0);
insert into part_ee_ff values ('EE', 0);
insert into list_parted values ('EE', 1);
insert into part_ee_ff values ('EE', 10);
insert into list_parted values ('aa'), ('cc');
insert into list_parted select 'Ff', s.a from generate_series(1, 29) s(a);
insert into list_parted select 'gg', s.a from generate_series(1, 9) s(a);
insert into list_parted (b) values (1);
insert into hash_parted values(generate_series(1,10));
insert into hpart0 values(12),(16);
insert into hpart0 values(11);
insert into hpart3 values(11);
insert into part_default values (null);
insert into part_default values (1);
insert into part_default values (-1);
insert into mlparted values (1, 2);
insert into mlparted (a, b) values (1, 5);
insert into mlparted values (1, 2);
insert into mlparted1 (a, b) values (2, 3);
insert into lparted_nonullpart values (1);
with ins (a, b, c) as
  (insert into mlparted (b, a) select s.a, 1 from generate_series(2, 39) s(a) returning tableoid::regclass, *)
  select a, b, min(c), max(c) from ins group by a, b order by 1;
insert into mlparted values (1, 45, 'a');
insert into mlparted5 (a, b, c) values (1, 40, 'a');
insert into mlparted values (40, 100);
insert into mlparted_def1 values (42, 100);
insert into mlparted_def2 values (54, 50);
insert into mlparted values (70, 100);
insert into mlparted_def1 values (52, 50);
insert into mlparted_def2 values (34, 50);
insert into mlparted values (70, 100);
insert into mlparted values (1, 2, 'a', 1);
insert into mlparted values (1, 40, 'a', 1);  -- goes to mlparted5_a
insert into mlparted values (1, 45, 'b', 1);  -- goes to mlparted5_b
insert into mlparted values (1, 45, 'c', 1);  -- goes to mlparted5_cd, fails
insert into mlparted values (1, 45, 'f', 1);  -- goes to mlparted5, fails
insert into mlparted values (1, 2, 'a', 1);
insert into mlparted values (1, 40, 'a', 1);  -- goes to mlparted5_a
insert into mlparted values (1, 45, 'b', 1);  -- goes to mlparted5_b
insert into mlparted values (1, 45, 'c', 1);  -- goes to mlparted5_cd, fails
insert into mlparted values (1, 45, 'f', 1);  -- goes to mlparted5, fails
insert into key_desc values (1, 1);
insert into key_desc values (1, 1);
insert into key_desc values (2, 1);
insert into mcrparted values (null, null, null);
insert into mcrparted values (0, 1, 1);
insert into mcrparted0 values (0, 1, 1);
insert into mcrparted values (9, 1000, 1);
insert into mcrparted1 values (9, 1000, 1);
insert into mcrparted values (10, 5, -1);
insert into mcrparted1 values (10, 5, -1);
insert into mcrparted values (2, 1, 0);
insert into mcrparted1 values (2, 1, 0);
insert into mcrparted values (10, 6, 1000);
insert into mcrparted2 values (10, 6, 1000);
insert into mcrparted values (10, 1000, 1000);
insert into mcrparted2 values (10, 1000, 1000);
insert into mcrparted values (11, 1, -1);
insert into mcrparted3 values (11, 1, -1);
insert into mcrparted values (30, 21, 20);
insert into mcrparted5 values (30, 21, 20);
insert into mcrparted4 values (30, 21, 20);	-- error
insert into brtrigpartcon values (1, 'hi there');
insert into brtrigpartcon1 values (1, 'hi there');
insert into donothingbrtrig_test values (1, 'foo'), (2, 'bar');
insert into mcrparted values ('aaa', 0), ('b', 0), ('bz', 10), ('c', -10),
    ('comm', -10), ('common', -10), ('common', 0), ('common', 10),
    ('commons', 0), ('d', -10), ('e', 0);
insert into returningwrtest values (1) returning returningwrtest;
insert into returningwrtest values (2, 'foo') returning returningwrtest;
