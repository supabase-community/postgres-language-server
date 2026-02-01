CREATE TYPE rainbow AS ENUM ('red', 'orange', 'yellow', 'green', 'blue', 'purple');

SELECT COUNT(*) FROM pg_enum WHERE enumtypid = 'rainbow'::regtype;

SELECT 'red'::rainbow;

SELECT 'mauve'::rainbow;

SELECT pg_input_is_valid('red', 'rainbow');

SELECT pg_input_is_valid('mauve', 'rainbow');

SELECT * FROM pg_input_error_info('mauve', 'rainbow');

SELECT * FROM pg_input_error_info(repeat('too_long', 32), 'rainbow');

CREATE TYPE dup_enum AS ENUM ('foo','bar','foo');

CREATE TYPE planets AS ENUM ( 'venus', 'earth', 'mars' );

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'planets'::regtype
ORDER BY 2;

ALTER TYPE planets ADD VALUE 'uranus';

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'planets'::regtype
ORDER BY 2;

ALTER TYPE planets ADD VALUE 'mercury' BEFORE 'venus';

ALTER TYPE planets ADD VALUE 'saturn' BEFORE 'uranus';

ALTER TYPE planets ADD VALUE 'jupiter' AFTER 'mars';

ALTER TYPE planets ADD VALUE 'neptune' AFTER 'uranus';

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'planets'::regtype
ORDER BY 2;

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'planets'::regtype
ORDER BY enumlabel::planets;

ALTER TYPE planets ADD VALUE
  'plutoplutoplutoplutoplutoplutoplutoplutoplutoplutoplutoplutoplutopluto';

ALTER TYPE planets ADD VALUE 'pluto' AFTER 'zeus';

ALTER TYPE planets ADD VALUE 'mercury';

ALTER TYPE planets ADD VALUE IF NOT EXISTS 'mercury';

SELECT enum_last(NULL::planets);

ALTER TYPE planets ADD VALUE IF NOT EXISTS 'pluto';

SELECT enum_last(NULL::planets);

create type insenum as enum ('L1', 'L2');

alter type insenum add value 'i1' before 'L2';

alter type insenum add value 'i2' before 'L2';

alter type insenum add value 'i3' before 'L2';

alter type insenum add value 'i4' before 'L2';

alter type insenum add value 'i5' before 'L2';

alter type insenum add value 'i6' before 'L2';

alter type insenum add value 'i7' before 'L2';

alter type insenum add value 'i8' before 'L2';

alter type insenum add value 'i9' before 'L2';

alter type insenum add value 'i10' before 'L2';

alter type insenum add value 'i11' before 'L2';

alter type insenum add value 'i12' before 'L2';

alter type insenum add value 'i13' before 'L2';

alter type insenum add value 'i14' before 'L2';

alter type insenum add value 'i15' before 'L2';

alter type insenum add value 'i16' before 'L2';

alter type insenum add value 'i17' before 'L2';

alter type insenum add value 'i18' before 'L2';

alter type insenum add value 'i19' before 'L2';

alter type insenum add value 'i20' before 'L2';

alter type insenum add value 'i21' before 'L2';

alter type insenum add value 'i22' before 'L2';

alter type insenum add value 'i23' before 'L2';

alter type insenum add value 'i24' before 'L2';

alter type insenum add value 'i25' before 'L2';

alter type insenum add value 'i26' before 'L2';

alter type insenum add value 'i27' before 'L2';

alter type insenum add value 'i28' before 'L2';

alter type insenum add value 'i29' before 'L2';

alter type insenum add value 'i30' before 'L2';

SELECT enumlabel,
       case when enumsortorder > 20 then null else enumsortorder end as so
FROM pg_enum
WHERE enumtypid = 'insenum'::regtype
ORDER BY enumsortorder;

CREATE TABLE enumtest (col rainbow);

INSERT INTO enumtest values ('red'), ('orange'), ('yellow'), ('green');

SELECT * FROM enumtest;

SELECT * FROM enumtest WHERE col = 'orange';

SELECT * FROM enumtest WHERE col <> 'orange' ORDER BY col;

SELECT * FROM enumtest WHERE col > 'yellow' ORDER BY col;

SELECT * FROM enumtest WHERE col >= 'yellow' ORDER BY col;

SELECT * FROM enumtest WHERE col < 'green' ORDER BY col;

SELECT * FROM enumtest WHERE col <= 'green' ORDER BY col;

SELECT 'red'::rainbow::text || 'hithere';

SELECT 'red'::text::rainbow = 'red'::rainbow;

SELECT min(col) FROM enumtest;

SELECT max(col) FROM enumtest;

SELECT max(col) FROM enumtest WHERE col < 'green';

SET enable_seqscan = off;

SET enable_bitmapscan = off;

CREATE UNIQUE INDEX enumtest_btree ON enumtest USING btree (col);

SELECT * FROM enumtest WHERE col = 'orange';

SELECT * FROM enumtest WHERE col <> 'orange' ORDER BY col;

SELECT * FROM enumtest WHERE col > 'yellow' ORDER BY col;

SELECT * FROM enumtest WHERE col >= 'yellow' ORDER BY col;

SELECT * FROM enumtest WHERE col < 'green' ORDER BY col;

SELECT * FROM enumtest WHERE col <= 'green' ORDER BY col;

SELECT min(col) FROM enumtest;

SELECT max(col) FROM enumtest;

SELECT max(col) FROM enumtest WHERE col < 'green';

DROP INDEX enumtest_btree;

CREATE INDEX enumtest_hash ON enumtest USING hash (col);

SELECT * FROM enumtest WHERE col = 'orange';

DROP INDEX enumtest_hash;

RESET enable_seqscan;

RESET enable_bitmapscan;

CREATE DOMAIN rgb AS rainbow CHECK (VALUE IN ('red', 'green', 'blue'));

SELECT 'red'::rgb;

SELECT 'purple'::rgb;

SELECT 'purple'::rainbow::rgb;

DROP DOMAIN rgb;

SELECT '{red,green,blue}'::rainbow[];

SELECT ('{red,green,blue}'::rainbow[])[2];

SELECT 'red' = ANY ('{red,green,blue}'::rainbow[]);

SELECT 'yellow' = ANY ('{red,green,blue}'::rainbow[]);

SELECT 'red' = ALL ('{red,green,blue}'::rainbow[]);

SELECT 'red' = ALL ('{red,red}'::rainbow[]);

SELECT enum_first(NULL::rainbow);

SELECT enum_last('green'::rainbow);

SELECT enum_range(NULL::rainbow);

SELECT enum_range('orange'::rainbow, 'green'::rainbow);

SELECT enum_range(NULL, 'green'::rainbow);

SELECT enum_range('orange'::rainbow, NULL);

SELECT enum_range(NULL::rainbow, NULL);

CREATE FUNCTION echo_me(anyenum) RETURNS text AS $$
BEGIN
RETURN $1::text || 'omg';
END
$$ LANGUAGE plpgsql;

SELECT echo_me('red'::rainbow);

CREATE FUNCTION echo_me(rainbow) RETURNS text AS $$
BEGIN
RETURN $1::text || 'wtf';
END
$$ LANGUAGE plpgsql;

SELECT echo_me('red'::rainbow);

DROP FUNCTION echo_me(anyenum);

SELECT echo_me('red');

DROP FUNCTION echo_me(rainbow);

CREATE TABLE enumtest_parent (id rainbow PRIMARY KEY);

CREATE TABLE enumtest_child (parent rainbow REFERENCES enumtest_parent);

INSERT INTO enumtest_parent VALUES ('red');

INSERT INTO enumtest_child VALUES ('red');

INSERT INTO enumtest_child VALUES ('blue');

DELETE FROM enumtest_parent;

CREATE TYPE bogus AS ENUM('good', 'bad', 'ugly');

CREATE TABLE enumtest_bogus_child(parent bogus REFERENCES enumtest_parent);

DROP TYPE bogus;

ALTER TYPE rainbow RENAME VALUE 'red' TO 'crimson';

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'rainbow'::regtype
ORDER BY 2;

ALTER TYPE rainbow RENAME VALUE 'red' TO 'crimson';

ALTER TYPE rainbow RENAME VALUE 'blue' TO 'green';

CREATE TYPE bogus AS ENUM('good');

BEGIN;

ALTER TYPE bogus ADD VALUE 'new';

SAVEPOINT x;

SELECT 'new'::bogus;

ROLLBACK TO x;

SELECT enum_first(null::bogus);

SELECT enum_last(null::bogus);

ROLLBACK TO x;

SELECT enum_range(null::bogus);

ROLLBACK TO x;

COMMIT;

SELECT 'new'::bogus;

SELECT enumlabel, enumsortorder
FROM pg_enum
WHERE enumtypid = 'bogus'::regtype
ORDER BY 2;

BEGIN;

ALTER TYPE bogus RENAME TO bogon;

ALTER TYPE bogon ADD VALUE 'bad';

SELECT 'bad'::bogon;

ROLLBACK;

BEGIN;

ALTER TYPE bogus RENAME VALUE 'good' to 'bad';

SELECT 'bad'::bogus;

ROLLBACK;

DROP TYPE bogus;

BEGIN;

CREATE TYPE bogus AS ENUM('good','bad','ugly');

ALTER TYPE bogus RENAME TO bogon;

select enum_range(null::bogon);

ROLLBACK;

BEGIN;

CREATE TYPE bogus AS ENUM('good');

ALTER TYPE bogus RENAME TO bogon;

ALTER TYPE bogon ADD VALUE 'bad';

ALTER TYPE bogon ADD VALUE 'ugly';

select enum_range(null::bogon);

ROLLBACK;

DROP TABLE enumtest_child;

DROP TABLE enumtest_parent;

DROP TABLE enumtest;

DROP TYPE rainbow;

SELECT COUNT(*) FROM pg_type WHERE typname = 'rainbow';

SELECT * FROM pg_enum WHERE NOT EXISTS
  (SELECT 1 FROM pg_type WHERE pg_type.oid = enumtypid);
