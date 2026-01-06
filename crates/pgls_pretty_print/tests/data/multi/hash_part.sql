CREATE TABLE mchash (a int, b text, c jsonb)
  PARTITION BY HASH (a part_test_int4_ops, b part_test_text_ops);

CREATE TABLE mchash1
  PARTITION OF mchash FOR VALUES WITH (MODULUS 4, REMAINDER 0);

SELECT satisfies_hash_partition(0, 4, 0, NULL);

SELECT satisfies_hash_partition('tenk1'::regclass, 4, 0, NULL);

SELECT satisfies_hash_partition('mchash1'::regclass, 4, 0, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, 0, 0, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, 1, -1, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, 1, 1, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, NULL, 0, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, 4, NULL, NULL);

SELECT satisfies_hash_partition('mchash'::regclass, 4, 0, NULL::int, NULL::text, NULL::json);

SELECT satisfies_hash_partition('mchash'::regclass, 3, 1, NULL::int);

SELECT satisfies_hash_partition('mchash'::regclass, 2, 1, NULL::int, NULL::int);

SELECT satisfies_hash_partition('mchash'::regclass, 4, 0, 0, ''::text);

SELECT satisfies_hash_partition('mchash'::regclass, 4, 0, 2, ''::text);

SELECT satisfies_hash_partition('mchash'::regclass, 2, 1,
								variadic array[1,2]::int[]);

CREATE TABLE mcinthash (a int, b int, c jsonb)
  PARTITION BY HASH (a part_test_int4_ops, b part_test_int4_ops);

SELECT satisfies_hash_partition('mcinthash'::regclass, 4, 0,
								variadic array[0, 0]);

SELECT satisfies_hash_partition('mcinthash'::regclass, 4, 0,
								variadic array[0, 1]);

SELECT satisfies_hash_partition('mcinthash'::regclass, 4, 0,
								variadic array[]::int[]);

SELECT satisfies_hash_partition('mcinthash'::regclass, 4, 0,
								variadic array[now(), now()]);

create table text_hashp (a text) partition by hash (a);

create table text_hashp0 partition of text_hashp for values with (modulus 2, remainder 0);

create table text_hashp1 partition of text_hashp for values with (modulus 2, remainder 1);

select satisfies_hash_partition('text_hashp'::regclass, 2, 0, 'xxx'::text) OR
	   satisfies_hash_partition('text_hashp'::regclass, 2, 1, 'xxx'::text) AS satisfies;

DROP TABLE mchash;

DROP TABLE mcinthash;

DROP TABLE text_hashp;
