SET datestyle TO ISO, YMD;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_rng_pk';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_rng_pk';

CREATE TABLE temporal_rng2 (LIKE temporal_rng INCLUDING ALL);

DROP TABLE temporal_rng2;

CREATE TABLE temporal_rng2 () INHERITS (temporal_rng);

DROP TABLE temporal_rng2;

DROP TABLE temporal_rng;

CREATE TABLE temporal_rng (
  id int4range,
  valid_at daterange
);

DROP TABLE temporal_rng CASCADE;

CREATE TABLE temporal_rng (
  id int4range,
  valid_at daterange
);

CREATE TABLE temporal_rng2 () INHERITS (temporal_rng);

DROP TABLE temporal_rng2;

DROP TABLE temporal_rng;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_rng2_pk';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_rng2_pk';

CREATE TYPE textrange2 AS range (subtype=text, collation="C");

ALTER TABLE temporal_rng3 DROP CONSTRAINT temporal_rng3_pk;

DROP TABLE temporal_rng3;

DROP TYPE textrange2;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_mltrng_pk';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_mltrng_pk';

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_mltrng2_pk';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_mltrng2_pk';

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_rng3_uq';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_rng3_uq';

DROP TABLE temporal_rng3;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_rng3_uq';

SELECT pg_get_indexdef(conindid, 0, true) FROM pg_constraint WHERE conname = 'temporal_rng3_uq';

DROP TABLE temporal_rng3;

CREATE TYPE textrange2 AS range (subtype=text, collation="C");

ALTER TABLE temporal_rng3 DROP CONSTRAINT temporal_rng3_uq;

DROP TABLE temporal_rng3;

DROP TYPE textrange2;

CREATE TABLE temporal_rng (
  id int4range,
  valid_at daterange
);

CREATE TABLE temporal3 (
  id int4range,
  valid_at daterange
);

CREATE INDEX idx_temporal3_uq ON temporal3 USING gist (id, valid_at);

ALTER TABLE temporal3
  ADD CONSTRAINT temporal3_pk
  PRIMARY KEY USING INDEX idx_temporal3_uq;

DROP TABLE temporal3;

CREATE TABLE temporal3 (
  id int4range,
  valid_at daterange
);

CREATE INDEX idx_temporal3_uq ON temporal3 USING gist (id, valid_at);

ALTER TABLE temporal3
  ADD CONSTRAINT temporal3_uq
  UNIQUE USING INDEX idx_temporal3_uq;

DROP TABLE temporal3;

CREATE TABLE temporal3 (
  id int4range,
  valid_at daterange
);

CREATE UNIQUE INDEX idx_temporal3_uq ON temporal3 (id, valid_at);

ALTER TABLE temporal3
  ADD CONSTRAINT temporal3_uq
  UNIQUE USING INDEX idx_temporal3_uq;

DROP TABLE temporal3;

CREATE TABLE temporal3 (
  id int4range
);

DROP TABLE temporal3;

CREATE TABLE temporal3 (
  id int4range
);

DROP TABLE temporal3;

ALTER TABLE temporal_rng DROP CONSTRAINT temporal_rng_pk;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-03'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-03-03', '2018-04-04'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[3,4)', daterange('2018-01-01', NULL));

ALTER TABLE temporal_rng DROP CONSTRAINT temporal_rng_pk;

BEGIN;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-01-01', '2018-01-05'));

ROLLBACK;

BEGIN;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[3,4)', 'empty');

ROLLBACK;

DELETE FROM temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-03'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-03-03', '2018-04-04'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[3,4)', daterange('2018-01-01', NULL));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng (id, valid_at) VALUES (NULL, daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[3,4)', NULL);

INSERT INTO temporal_rng (id, valid_at) VALUES ('[3,4)', 'empty');

SELECT * FROM temporal_rng ORDER BY id, valid_at;

UPDATE  temporal_rng
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_rng
SET     valid_at = '[2020-01-01,2021-01-01)'
WHERE   id = '[11,12)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_rng
SET     id = '[21,22)',
        valid_at = '[2018-01-02,2018-02-03)'
WHERE   id = '[11,12)'
AND     valid_at @> '2020-01-15'::date;

SELECT * FROM temporal_rng ORDER BY id, valid_at;

UPDATE  temporal_rng
SET     id = '[1,2)',
        valid_at = daterange('2018-03-05', '2018-05-05')
WHERE   id = '[21,22)';

UPDATE  temporal_rng
SET     id = NULL,
        valid_at = daterange('2018-03-05', '2018-05-05')
WHERE   id = '[21,22)';

UPDATE  temporal_rng
SET     id = '[1,2)',
        valid_at = NULL
WHERE   id = '[21,22)';

UPDATE  temporal_rng
SET     id = '[1,2)',
        valid_at = 'empty'
WHERE   id = '[21,22)';

SELECT * FROM temporal_rng ORDER BY id, valid_at;

CREATE TABLE temporal_rng3 (
  id int4range,
  valid_at daterange
);

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-03'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-03-03', '2018-04-04'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[2,3)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', daterange('2018-01-01', NULL));

INSERT INTO temporal_rng3 (id, valid_at) VALUES (NULL, daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', NULL);

ALTER TABLE temporal_rng3 DROP CONSTRAINT temporal_rng3_uq;

BEGIN;

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-01-01', '2018-01-05'));

ROLLBACK;

BEGIN;

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', 'empty');

ROLLBACK;

DELETE FROM temporal_rng3;

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-03'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-03-03', '2018-04-04'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[2,3)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', daterange('2018-01-01', NULL));

INSERT INTO temporal_rng3 (id, valid_at) VALUES (NULL, daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', NULL);

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[1,2)', daterange('2018-01-01', '2018-01-05'));

INSERT INTO temporal_rng3 (id, valid_at) VALUES ('[3,4)', 'empty');

SELECT * FROM temporal_rng3 ORDER BY id, valid_at;

UPDATE  temporal_rng3
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_rng3
SET     valid_at = '[2020-01-01,2021-01-01)'
WHERE   id = '[11,12)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_rng3
SET     id = '[21,22)',
        valid_at = '[2018-01-02,2018-02-03)'
WHERE   id = '[11,12)'
AND     valid_at @> '2020-01-15'::date;

UPDATE  temporal_rng3
SET     id = NULL,
        valid_at = daterange('2020-01-01', '2021-01-01')
WHERE   id = '[21,22)';

UPDATE  temporal_rng3
SET     id = '[1,2)',
        valid_at = NULL
WHERE   id IS NULL AND valid_at @> '2020-06-01'::date;

SELECT * FROM temporal_rng3 ORDER BY id, valid_at;

UPDATE  temporal_rng3
SET     valid_at = daterange('2018-03-01', '2018-05-05')
WHERE   id = '[1,2)' AND valid_at IS NULL;

UPDATE  temporal_rng3
SET     valid_at = 'empty'
WHERE   id = '[1,2)' AND valid_at IS NULL;

UPDATE  temporal_rng3
SET     id = NULL,
        valid_at = 'empty'
WHERE   id = '[1,2)' AND valid_at IS NULL;

SELECT * FROM temporal_rng3 ORDER BY id, valid_at;

DROP TABLE temporal_rng3;

ALTER TABLE temporal_mltrng DROP CONSTRAINT temporal_mltrng_pk;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-03')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-03-03', '2018-04-04')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[3,4)', datemultirange(daterange('2018-01-01', NULL)));

ALTER TABLE temporal_mltrng DROP CONSTRAINT temporal_mltrng_pk;

BEGIN;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-01', '2018-01-05')));

ROLLBACK;

BEGIN;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[3,4)', '{}');

ROLLBACK;

DELETE FROM temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-03')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-03-03', '2018-04-04')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[3,4)', datemultirange(daterange('2018-01-01', NULL)));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES (NULL, datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[3,4)', NULL);

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[3,4)', '{}');

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

UPDATE  temporal_mltrng
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_mltrng
SET     valid_at = '{[2020-01-01,2021-01-01)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_mltrng
SET     id = '[21,22)',
        valid_at = '{[2018-01-02,2018-02-03)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2020-01-15'::date;

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

UPDATE  temporal_mltrng
SET     id = '[1,2)',
        valid_at = datemultirange(daterange('2018-03-05', '2018-05-05'))
WHERE   id = '[21,22)';

UPDATE  temporal_mltrng
SET     id = NULL,
        valid_at = datemultirange(daterange('2018-03-05', '2018-05-05'))
WHERE   id = '[21,22)';

UPDATE  temporal_mltrng
SET     id = '[1,2)',
        valid_at = NULL
WHERE   id = '[21,22)';

UPDATE  temporal_mltrng
SET     id = '[1,2)',
        valid_at = '{}'
WHERE   id = '[21,22)';

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

CREATE TABLE temporal_mltrng3 (
  id int4range,
  valid_at datemultirange
);

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-03')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-03-03', '2018-04-04')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', datemultirange(daterange('2018-01-01', NULL)));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES (NULL, datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', NULL);

ALTER TABLE temporal_mltrng3 DROP CONSTRAINT temporal_mltrng3_uq;

BEGIN;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-01', '2018-01-05')));

ROLLBACK;

BEGIN;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', '{}');

ROLLBACK;

DELETE FROM temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-03')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-03-03', '2018-04-04')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', datemultirange(daterange('2018-01-01', NULL)));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES (NULL, datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', NULL);

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-01-01', '2018-01-05')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[3,4)', '{}');

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

UPDATE  temporal_mltrng3
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_mltrng3
SET     valid_at = '{[2020-01-01,2021-01-01)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2018-01-15'::date;

UPDATE  temporal_mltrng3
SET     id = '[21,22)',
        valid_at = '{[2018-01-02,2018-02-03)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2020-01-15'::date;

UPDATE  temporal_mltrng3
SET     id = NULL,
        valid_at = datemultirange(daterange('2020-01-01', '2021-01-01'))
WHERE   id = '[21,22)';

UPDATE  temporal_mltrng3
SET     id = '[1,2)',
        valid_at = NULL
WHERE   id IS NULL AND valid_at @> '2020-06-01'::date;

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

UPDATE  temporal_mltrng3
SET     valid_at = datemultirange(daterange('2018-03-01', '2018-05-05'))
WHERE   id = '[1,2)' AND valid_at IS NULL;

UPDATE  temporal_mltrng3
SET     valid_at = '{}'
WHERE   id = '[1,2)' AND valid_at IS NULL;

UPDATE  temporal_mltrng3
SET     id = NULL,
        valid_at = '{}'
WHERE   id = '[1,2)' AND valid_at IS NULL;

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

DROP TABLE temporal_mltrng3;

INSERT INTO temporal3 (id, valid_at, id2, name)
  VALUES
  ('[1,2)', daterange('2000-01-01', '2010-01-01'), '[7,8)', 'foo'),
  ('[2,3)', daterange('2000-01-01', '2010-01-01'), '[9,10)', 'bar')
;

DROP TABLE temporal3;

ALTER TABLE temporal3 ALTER COLUMN valid_at DROP NOT NULL;

ALTER TABLE temporal3 ALTER COLUMN valid_at TYPE tstzrange USING tstzrange(lower(valid_at), upper(valid_at));

ALTER TABLE temporal3 RENAME COLUMN valid_at TO valid_thru;

ALTER TABLE temporal3 DROP COLUMN valid_thru;

DROP TABLE temporal3;

CREATE TABLE tp1 PARTITION OF temporal_partitioned FOR VALUES IN ('[1,2)', '[2,3)');

CREATE TABLE tp2 PARTITION OF temporal_partitioned FOR VALUES IN ('[3,4)', '[4,5)');

INSERT INTO temporal_partitioned (id, valid_at, name) VALUES
  ('[1,2)', daterange('2000-01-01', '2000-02-01'), 'one'),
  ('[1,2)', daterange('2000-02-01', '2000-03-01'), 'one'),
  ('[3,4)', daterange('2000-01-01', '2010-01-01'), 'three');

SELECT * FROM temporal_partitioned ORDER BY id, valid_at;

SELECT * FROM tp1 ORDER BY id, valid_at;

SELECT * FROM tp2 ORDER BY id, valid_at;

DROP TABLE temporal_partitioned;

CREATE TABLE tp1 PARTITION OF temporal_partitioned FOR VALUES IN ('[1,2)', '[2,3)');

CREATE TABLE tp2 PARTITION OF temporal_partitioned FOR VALUES IN ('[3,4)', '[4,5)');

INSERT INTO temporal_partitioned (id, valid_at, name) VALUES
  ('[1,2)', daterange('2000-01-01', '2000-02-01'), 'one'),
  ('[1,2)', daterange('2000-02-01', '2000-03-01'), 'one'),
  ('[3,4)', daterange('2000-01-01', '2010-01-01'), 'three');

SELECT * FROM temporal_partitioned ORDER BY id, valid_at;

SELECT * FROM tp1 ORDER BY id, valid_at;

SELECT * FROM tp2 ORDER BY id, valid_at;

DROP TABLE temporal_partitioned;

ALTER TABLE temporal_rng REPLICA IDENTITY USING INDEX temporal_rng_pk;

TRUNCATE temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT DO NOTHING;

SELECT * FROM temporal_rng ORDER BY id, valid_at;

TRUNCATE temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

SELECT * FROM temporal_rng ORDER BY id, valid_at;

TRUNCATE temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO NOTHING;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO NOTHING;

SELECT * FROM temporal_rng ORDER BY id, valid_at;

TRUNCATE temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_rng ORDER BY id, valid_at;

TRUNCATE temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_rng (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_rng ORDER BY id, valid_at;

TRUNCATE temporal3;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT DO NOTHING;

SELECT * FROM temporal3 ORDER BY id, valid_at;

TRUNCATE temporal3;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO NOTHING;

SELECT * FROM temporal3 ORDER BY id, valid_at;

TRUNCATE temporal3;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO NOTHING;

INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO NOTHING;

SELECT * FROM temporal3 ORDER BY id, valid_at;

TRUNCATE temporal3;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal3 ORDER BY id, valid_at;

TRUNCATE temporal3;

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2000-01-01', '2010-01-01'));

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal3 ORDER BY id, valid_at;

DROP TABLE temporal3;

TRUNCATE temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT DO NOTHING;

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

TRUNCATE temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

TRUNCATE temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO NOTHING;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO NOTHING;

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

TRUNCATE temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

TRUNCATE temporal_mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_mltrng ORDER BY id, valid_at;

TRUNCATE temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT DO NOTHING;

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

TRUNCATE temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO NOTHING;

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

TRUNCATE temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO NOTHING;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO NOTHING;

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

TRUNCATE temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

TRUNCATE temporal_mltrng3;

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2000-01-01', '2010-01-01')));

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO UPDATE SET id = EXCLUDED.id + '[2,3)';

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO UPDATE SET id = EXCLUDED.id + '[3,4)';

INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[2,3)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng3_uq DO UPDATE SET id = EXCLUDED.id + '[4,5)';

SELECT * FROM temporal_mltrng3 ORDER BY id, valid_at;

DROP TABLE temporal_mltrng3;

ALTER TABLE temporal3 DROP COLUMN valid_at;

ALTER TABLE temporal3 DROP COLUMN valid_at CASCADE;

DROP TABLE temporal_fk_rng2rng;

DROP TABLE temporal3;

DROP TABLE temporal_rng;

CREATE TABLE temporal_rng (id int4range, valid_at daterange);

DROP TABLE temporal_fk_rng2rng;

DROP TABLE temporal_fk_rng2rng;

DROP TABLE temporal_rng2;

DROP TABLE temporal_fk2_rng2rng;

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk,
  ALTER COLUMN valid_at TYPE tsrange USING tsrange(lower(valid_at), upper(valid_at));

ALTER TABLE temporal_fk_rng2rng
  ALTER COLUMN valid_at TYPE daterange USING daterange(lower(valid_at)::date, upper(valid_at)::date);

DELETE FROM temporal_fk_rng2rng;

DELETE FROM temporal_rng;

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[1,2)', daterange('2018-01-02', '2018-02-03')),
  ('[1,2)', daterange('2018-03-03', '2018-04-04')),
  ('[2,3)', daterange('2018-01-01', '2018-01-05')),
  ('[3,4)', daterange('2018-01-01', NULL));

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk;

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-01'), '[1,2)');

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk;

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[2,3)', daterange('2018-01-02', '2018-04-01'), '[1,2)');

DELETE FROM temporal_fk_rng2rng;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_fk_rng2rng_fk';

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[1,2)', daterange('2018-01-02', '2018-02-01'), '[1,2)');

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[2,3)', daterange('2018-01-02', '2018-04-01'), '[1,2)');

INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2018-02-03', '2018-03-03'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[2,3)', daterange('2018-01-02', '2018-04-01'), '[1,2)');

UPDATE temporal_fk_rng2rng SET valid_at = daterange('2018-01-02', '2018-02-20') WHERE id = '[1,2)';

UPDATE temporal_fk_rng2rng SET valid_at = daterange('2018-01-02', '2018-05-01') WHERE id = '[1,2)';

UPDATE temporal_fk_rng2rng SET parent_id = '[8,9)' WHERE id = '[1,2)';

BEGIN;

INSERT INTO temporal_rng (id, valid_at) VALUES
    ('[5,6)', daterange('2018-01-01', '2018-02-01')),
    ('[5,6)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
    ('[3,4)', daterange('2018-01-05', '2018-01-10'), '[5,6)');

ALTER TABLE temporal_fk_rng2rng
    ALTER CONSTRAINT temporal_fk_rng2rng_fk
    DEFERRABLE INITIALLY DEFERRED;

DELETE FROM temporal_rng WHERE id = '[5,6)';

COMMIT;

TRUNCATE temporal_rng, temporal_fk_rng2rng;

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[5,6)', daterange('2018-01-01', '2018-02-01'));

UPDATE temporal_rng SET valid_at = daterange('2016-01-01', '2016-02-01') WHERE id = '[5,6)';

DELETE FROM temporal_rng WHERE id = '[5,6)';

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[5,6)', daterange('2018-01-01', '2018-02-01')),
  ('[5,6)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id)
  VALUES ('[3,4)', daterange('2018-01-05', '2018-01-10'), '[5,6)');

UPDATE temporal_rng SET valid_at = daterange('2016-02-01', '2016-03-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-02-01', '2018-03-01');

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[6,7)', daterange('2018-01-01', '2018-02-01')),
  ('[6,7)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[4,5)', daterange('2018-01-15', '2018-02-15'), '[6,7)');

UPDATE temporal_rng
  SET valid_at = CASE WHEN lower(valid_at) = '2018-01-01' THEN daterange('2018-01-01', '2018-01-05')
                      WHEN lower(valid_at) = '2018-02-01' THEN daterange('2018-01-05', '2018-03-01') END
  WHERE id = '[6,7)';

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[1,2)', daterange('2018-01-01', '2018-03-01')),
  ('[1,2)', daterange('2018-03-01', '2018-06-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[1,2)', daterange('2018-01-15', '2018-02-01'), '[1,2)'),
  ('[2,3)', daterange('2018-01-15', '2018-05-01'), '[1,2)');

UPDATE temporal_rng SET valid_at = daterange('2018-01-15', '2018-03-01')
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

UPDATE temporal_rng SET valid_at = daterange('2018-01-01', '2018-03-01')
  WHERE id = '[1,2)' AND valid_at @> '2018-01-25'::date;

UPDATE temporal_rng SET id = '[2,3)', valid_at = daterange('2018-01-15', '2018-03-01')
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

UPDATE temporal_rng SET id = '[2,3)'
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[2,3)', daterange('2018-01-01', '2018-03-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[5,6)', daterange('2018-01-15', '2018-02-01'), '[2,3)');

UPDATE temporal_rng SET valid_at = daterange('2018-01-15', '2018-02-15')
  WHERE id = '[2,3)';

UPDATE temporal_rng SET valid_at = daterange('2016-01-01', '2016-02-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

BEGIN;

ALTER TABLE temporal_fk_rng2rng
    ALTER CONSTRAINT temporal_fk_rng2rng_fk
    DEFERRABLE INITIALLY DEFERRED;

UPDATE temporal_rng SET valid_at = daterange('2016-01-01', '2016-02-01')
    WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

COMMIT;

UPDATE temporal_rng SET id = '[7,8)'
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

DELETE FROM temporal_fk_rng2rng WHERE id = '[3,4)';

UPDATE temporal_rng SET valid_at = daterange('2016-01-01', '2016-02-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

TRUNCATE temporal_rng, temporal_fk_rng2rng;

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk;

TRUNCATE temporal_rng, temporal_fk_rng2rng;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[5,6)', daterange('2018-01-01', '2018-02-01'));

DELETE FROM temporal_rng WHERE id = '[5,6)';

INSERT INTO temporal_rng (id, valid_at) VALUES
  ('[5,6)', daterange('2018-01-01', '2018-02-01')),
  ('[5,6)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[3,4)', daterange('2018-01-05', '2018-01-10'), '[5,6)');

DELETE FROM temporal_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-02-01', '2018-03-01');

DELETE FROM temporal_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

BEGIN;

ALTER TABLE temporal_fk_rng2rng
    ALTER CONSTRAINT temporal_fk_rng2rng_fk
    DEFERRABLE INITIALLY DEFERRED;

DELETE FROM temporal_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

COMMIT;

DELETE FROM temporal_fk_rng2rng WHERE id = '[3,4)';

DELETE FROM temporal_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

TRUNCATE temporal_rng, temporal_fk_rng2rng;

ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk;

INSERT INTO temporal_rng (id, valid_at) VALUES ('[6,7)', daterange('2018-01-01', '2021-01-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[4,5)', daterange('2018-01-01', '2021-01-01'), '[6,7)');

INSERT INTO temporal_rng (id, valid_at) VALUES ('[9,10)', daterange('2018-01-01', '2021-01-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[6,7)', daterange('2018-01-01', '2021-01-01'), '[9,10)');

INSERT INTO temporal_rng (id, valid_at) VALUES ('[-1,-1]', daterange(null, null));

INSERT INTO temporal_rng (id, valid_at) VALUES ('[12,13)', daterange('2018-01-01', '2021-01-01'));

INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES ('[8,9)', daterange('2018-01-01', '2021-01-01'), '[12,13)');

DROP TABLE temporal_mltrng;

CREATE TABLE temporal_mltrng ( id int4range, valid_at datemultirange);

DROP TABLE temporal_fk_mltrng2mltrng;

DROP TABLE temporal_fk_mltrng2mltrng;

DROP TABLE temporal_mltrng2;

DROP TABLE temporal_fk2_mltrng2mltrng;

DELETE FROM temporal_fk_mltrng2mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-03'))),
  ('[1,2)', datemultirange(daterange('2018-03-03', '2018-04-04'))),
  ('[2,3)', datemultirange(daterange('2018-01-01', '2018-01-05'))),
  ('[3,4)', datemultirange(daterange('2018-01-01', NULL)));

ALTER TABLE temporal_fk_mltrng2mltrng
  DROP CONSTRAINT temporal_fk_mltrng2mltrng_fk;

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-01')), '[1,2)');

ALTER TABLE temporal_fk_mltrng2mltrng
  DROP CONSTRAINT temporal_fk_mltrng2mltrng_fk;

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[2,3)', datemultirange(daterange('2018-01-02', '2018-04-01')), '[1,2)');

DELETE FROM temporal_fk_mltrng2mltrng;

SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'temporal_fk_mltrng2mltrng_fk';

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[1,2)', datemultirange(daterange('2018-01-02', '2018-02-01')), '[1,2)');

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[2,3)', datemultirange(daterange('2018-01-02', '2018-04-01')), '[1,2)');

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2018-02-03', '2018-03-03')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[2,3)', datemultirange(daterange('2018-01-02', '2018-04-01')), '[1,2)');

UPDATE temporal_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2018-01-02', '2018-02-20')) WHERE id = '[1,2)';

UPDATE temporal_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2018-01-02', '2018-05-01')) WHERE id = '[1,2)';

UPDATE temporal_fk_mltrng2mltrng SET parent_id = '[8,9)' WHERE id = '[1,2)';

BEGIN;

INSERT INTO temporal_mltrng (id, valid_at) VALUES
    ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01'))),
    ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
    ('[3,4)', datemultirange(daterange('2018-01-05', '2018-01-10')), '[5,6)');

ALTER TABLE temporal_fk_mltrng2mltrng
    ALTER CONSTRAINT temporal_fk_mltrng2mltrng_fk
    DEFERRABLE INITIALLY DEFERRED;

DELETE FROM temporal_mltrng WHERE id = '[5,6)';

COMMIT;

TRUNCATE temporal_mltrng, temporal_fk_mltrng2mltrng;

ALTER TABLE temporal_fk_mltrng2mltrng
  DROP CONSTRAINT temporal_fk_mltrng2mltrng_fk;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01')));

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2016-01-01', '2016-02-01')) WHERE id = '[5,6)';

DELETE FROM temporal_mltrng WHERE id = '[5,6)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01'))),
  ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[3,4)', datemultirange(daterange('2018-01-05', '2018-01-10')), '[5,6)');

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2016-02-01', '2016-03-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[6,7)', datemultirange(daterange('2018-01-01', '2018-02-01'))),
  ('[6,7)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[4,5)', datemultirange(daterange('2018-01-15', '2018-02-15')), '[6,7)');

UPDATE temporal_mltrng
  SET valid_at = CASE WHEN lower(valid_at) = '2018-01-01' THEN datemultirange(daterange('2018-01-01', '2018-01-05'))
                      WHEN lower(valid_at) = '2018-02-01' THEN datemultirange(daterange('2018-01-05', '2018-03-01')) END
  WHERE id = '[6,7)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[1,2)', datemultirange(daterange('2018-01-01', '2018-03-01'))),
  ('[1,2)', datemultirange(daterange('2018-03-01', '2018-06-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[1,2)', datemultirange(daterange('2018-01-15', '2018-02-01')), '[1,2)'),
  ('[2,3)', datemultirange(daterange('2018-01-15', '2018-05-01')), '[1,2)');

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2018-01-15', '2018-03-01'))
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2018-01-01', '2018-03-01'))
  WHERE id = '[1,2)' AND valid_at @> '2018-01-25'::date;

UPDATE temporal_mltrng SET id = '[2,3)', valid_at = datemultirange(daterange('2018-01-15', '2018-03-01'))
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

UPDATE temporal_mltrng SET id = '[2,3)'
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[2,3)', datemultirange(daterange('2018-01-01', '2018-03-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[5,6)', datemultirange(daterange('2018-01-15', '2018-02-01')), '[2,3)');

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2018-01-15', '2018-02-15'))
  WHERE id = '[2,3)';

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2016-01-01', '2016-02-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

BEGIN;

ALTER TABLE temporal_fk_mltrng2mltrng
    ALTER CONSTRAINT temporal_fk_mltrng2mltrng_fk
    DEFERRABLE INITIALLY DEFERRED;

UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2016-01-01', '2016-02-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

COMMIT;

UPDATE temporal_mltrng SET id = '[7,8)'
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

TRUNCATE temporal_mltrng, temporal_fk_mltrng2mltrng;

ALTER TABLE temporal_fk_mltrng2mltrng
  DROP CONSTRAINT temporal_fk_mltrng2mltrng_fk;

TRUNCATE temporal_mltrng, temporal_fk_mltrng2mltrng;

INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01')));

DELETE FROM temporal_mltrng WHERE id = '[5,6)';

INSERT INTO temporal_mltrng (id, valid_at) VALUES
  ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01'))),
  ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[3,4)', datemultirange(daterange('2018-01-05', '2018-01-10')), '[5,6)');

DELETE FROM temporal_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-02-01', '2018-03-01'));

DELETE FROM temporal_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

BEGIN;

ALTER TABLE temporal_fk_mltrng2mltrng
    ALTER CONSTRAINT temporal_fk_mltrng2mltrng_fk
    DEFERRABLE INITIALLY DEFERRED;

DELETE FROM temporal_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

COMMIT;

CREATE TABLE tp1 partition OF temporal_partitioned_rng FOR VALUES IN ('[1,2)', '[3,4)', '[5,6)', '[7,8)', '[9,10)', '[11,12)');

CREATE TABLE tp2 partition OF temporal_partitioned_rng FOR VALUES IN ('[2,3)', '[4,5)', '[6,7)', '[8,9)', '[10,11)', '[12,13)');

INSERT INTO temporal_partitioned_rng (id, valid_at, name) VALUES
  ('[1,2)', daterange('2000-01-01', '2000-02-01'), 'one'),
  ('[1,2)', daterange('2000-02-01', '2000-03-01'), 'one'),
  ('[2,3)', daterange('2000-01-01', '2010-01-01'), 'two');

CREATE TABLE tfkp1 partition OF temporal_partitioned_fk_rng2rng FOR VALUES IN ('[1,2)', '[3,4)', '[5,6)', '[7,8)', '[9,10)', '[11,12)');

CREATE TABLE tfkp2 partition OF temporal_partitioned_fk_rng2rng FOR VALUES IN ('[2,3)', '[4,5)', '[6,7)', '[8,9)', '[10,11)', '[12,13)');

INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[1,2)', daterange('2000-01-01', '2000-02-15'), '[1,2)'),
  ('[1,2)', daterange('2001-01-01', '2002-01-01'), '[2,3)'),
  ('[2,3)', daterange('2000-01-01', '2000-02-15'), '[1,2)');

INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[3,4)', daterange('2010-01-01', '2010-02-15'), '[1,2)');

INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[3,4)', daterange('2000-01-01', '2000-02-15'), '[3,4)');

UPDATE temporal_partitioned_fk_rng2rng SET valid_at = daterange('2000-01-01', '2000-02-13') WHERE id = '[2,3)';

UPDATE temporal_partitioned_fk_rng2rng SET id = '[4,5)' WHERE id = '[1,2)';

UPDATE temporal_partitioned_fk_rng2rng SET id = '[1,2)' WHERE id = '[4,5)';

UPDATE temporal_partitioned_fk_rng2rng SET valid_at = daterange('2000-01-01', '2000-04-01') WHERE id = '[1,2)';

TRUNCATE temporal_partitioned_rng, temporal_partitioned_fk_rng2rng;

INSERT INTO temporal_partitioned_rng (id, valid_at) VALUES ('[5,6)', daterange('2016-01-01', '2016-02-01'));

UPDATE temporal_partitioned_rng SET valid_at = daterange('2018-01-01', '2018-02-01') WHERE id = '[5,6)';

INSERT INTO temporal_partitioned_rng (id, valid_at) VALUES ('[5,6)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES ('[3,4)', daterange('2018-01-05', '2018-01-10'), '[5,6)');

UPDATE temporal_partitioned_rng SET valid_at = daterange('2016-02-01', '2016-03-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-02-01', '2018-03-01');

UPDATE temporal_partitioned_rng SET valid_at = daterange('2016-01-01', '2016-02-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

TRUNCATE temporal_partitioned_rng, temporal_partitioned_fk_rng2rng;

INSERT INTO temporal_partitioned_rng (id, valid_at) VALUES ('[5,6)', daterange('2018-01-01', '2018-02-01'));

INSERT INTO temporal_partitioned_rng (id, valid_at) VALUES ('[5,6)', daterange('2018-02-01', '2018-03-01'));

INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES ('[3,4)', daterange('2018-01-05', '2018-01-10'), '[5,6)');

DELETE FROM temporal_partitioned_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-02-01', '2018-03-01');

DELETE FROM temporal_partitioned_rng WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');

DROP TABLE temporal_partitioned_fk_rng2rng;

DROP TABLE temporal_partitioned_rng;

CREATE TABLE tp1 PARTITION OF temporal_partitioned_mltrng FOR VALUES IN ('[1,2)', '[3,4)', '[5,6)', '[7,8)', '[9,10)', '[11,12)', '[13,14)', '[15,16)', '[17,18)', '[19,20)', '[21,22)', '[23,24)');

CREATE TABLE tp2 PARTITION OF temporal_partitioned_mltrng FOR VALUES IN ('[0,1)', '[2,3)', '[4,5)', '[6,7)', '[8,9)', '[10,11)', '[12,13)', '[14,15)', '[16,17)', '[18,19)', '[20,21)', '[22,23)', '[24,25)');

INSERT INTO temporal_partitioned_mltrng (id, valid_at, name) VALUES
  ('[1,2)', datemultirange(daterange('2000-01-01', '2000-02-01')), 'one'),
  ('[1,2)', datemultirange(daterange('2000-02-01', '2000-03-01')), 'one'),
  ('[2,3)', datemultirange(daterange('2000-01-01', '2010-01-01')), 'two');

CREATE TABLE tfkp1 PARTITION OF temporal_partitioned_fk_mltrng2mltrng FOR VALUES IN ('[1,2)', '[3,4)', '[5,6)', '[7,8)', '[9,10)', '[11,12)', '[13,14)', '[15,16)', '[17,18)', '[19,20)', '[21,22)', '[23,24)');

CREATE TABLE tfkp2 PARTITION OF temporal_partitioned_fk_mltrng2mltrng FOR VALUES IN ('[0,1)', '[2,3)', '[4,5)', '[6,7)', '[8,9)', '[10,11)', '[12,13)', '[14,15)', '[16,17)', '[18,19)', '[20,21)', '[22,23)', '[24,25)');

INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[1,2)', datemultirange(daterange('2000-01-01', '2000-02-15')), '[1,2)'),
  ('[1,2)', datemultirange(daterange('2001-01-01', '2002-01-01')), '[2,3)'),
  ('[2,3)', datemultirange(daterange('2000-01-01', '2000-02-15')), '[1,2)');

INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[3,4)', datemultirange(daterange('2010-01-01', '2010-02-15')), '[1,2)');

INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[3,4)', datemultirange(daterange('2000-01-01', '2000-02-15')), '[3,4)');

UPDATE temporal_partitioned_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2000-01-01', '2000-02-13')) WHERE id = '[2,3)';

UPDATE temporal_partitioned_fk_mltrng2mltrng SET id = '[4,5)' WHERE id = '[1,2)';

UPDATE temporal_partitioned_fk_mltrng2mltrng SET id = '[1,2)' WHERE id = '[4,5)';

UPDATE temporal_partitioned_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2000-01-01', '2000-04-01')) WHERE id = '[1,2)';

TRUNCATE temporal_partitioned_mltrng, temporal_partitioned_fk_mltrng2mltrng;

INSERT INTO temporal_partitioned_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2016-01-01', '2016-02-01')));

UPDATE temporal_partitioned_mltrng SET valid_at = datemultirange(daterange('2018-01-01', '2018-02-01')) WHERE id = '[5,6)';

INSERT INTO temporal_partitioned_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[3,4)', datemultirange(daterange('2018-01-05', '2018-01-10')), '[5,6)');

UPDATE temporal_partitioned_mltrng SET valid_at = datemultirange(daterange('2016-02-01', '2016-03-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-02-01', '2018-03-01'));

UPDATE temporal_partitioned_mltrng SET valid_at = datemultirange(daterange('2016-01-01', '2016-02-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

TRUNCATE temporal_partitioned_mltrng, temporal_partitioned_fk_mltrng2mltrng;

INSERT INTO temporal_partitioned_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01')));

INSERT INTO temporal_partitioned_mltrng (id, valid_at) VALUES ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));

INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES ('[3,4)', datemultirange(daterange('2018-01-05', '2018-01-10')), '[5,6)');

DELETE FROM temporal_partitioned_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-02-01', '2018-03-01'));

DELETE FROM temporal_partitioned_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));

DROP TABLE temporal_partitioned_fk_mltrng2mltrng;

DROP TABLE temporal_partitioned_mltrng;

RESET datestyle;
