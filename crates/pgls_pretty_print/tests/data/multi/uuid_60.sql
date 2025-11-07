CREATE TABLE guid1
(
	guid_field UUID,
	text_field TEXT DEFAULT(now())
);

CREATE TABLE guid2
(
	guid_field UUID,
	text_field TEXT DEFAULT(now())
);

CREATE TABLE guid3
(
	id SERIAL,
	guid_field UUID
);

INSERT INTO guid1(guid_field) VALUES('11111111-1111-1111-1111-111111111111F');

INSERT INTO guid1(guid_field) VALUES('{11111111-1111-1111-1111-11111111111}');

INSERT INTO guid1(guid_field) VALUES('111-11111-1111-1111-1111-111111111111');

INSERT INTO guid1(guid_field) VALUES('{22222222-2222-2222-2222-222222222222 ');

INSERT INTO guid1(guid_field) VALUES('11111111-1111-1111-G111-111111111111');

INSERT INTO guid1(guid_field) VALUES('11+11111-1111-1111-1111-111111111111');

SELECT pg_input_is_valid('11', 'uuid');

SELECT * FROM pg_input_error_info('11', 'uuid');

INSERT INTO guid1(guid_field) VALUES('11111111-1111-1111-1111-111111111111');

INSERT INTO guid1(guid_field) VALUES('{22222222-2222-2222-2222-222222222222}');

INSERT INTO guid1(guid_field) VALUES('3f3e3c3b3a3039383736353433a2313e');

SELECT guid_field FROM guid1;

SELECT guid_field FROM guid1 ORDER BY guid_field ASC;

SELECT guid_field FROM guid1 ORDER BY guid_field DESC;

SELECT COUNT(*) FROM guid1 WHERE guid_field = '3f3e3c3b-3a30-3938-3736-353433a2313e';

SELECT COUNT(*) FROM guid1 WHERE guid_field <> '11111111111111111111111111111111';

SELECT COUNT(*) FROM guid1 WHERE guid_field < '22222222-2222-2222-2222-222222222222';

SELECT COUNT(*) FROM guid1 WHERE guid_field <= '22222222-2222-2222-2222-222222222222';

SELECT COUNT(*) FROM guid1 WHERE guid_field > '22222222-2222-2222-2222-222222222222';

SELECT COUNT(*) FROM guid1 WHERE guid_field >= '22222222-2222-2222-2222-222222222222';

CREATE INDEX guid1_btree ON guid1 USING BTREE (guid_field);

CREATE INDEX guid1_hash  ON guid1 USING HASH  (guid_field);

CREATE UNIQUE INDEX guid1_unique_BTREE ON guid1 USING BTREE (guid_field);

SELECT COUNT(*) FROM guid1 WHERE guid_field <> '11111111111111111111111111111111' OR
							guid_field <> '3f3e3c3b-3a30-3938-3736-353433a2313e';

SELECT COUNT(*) FROM guid1 WHERE guid_field <= '22222222-2222-2222-2222-222222222222' OR
									guid_field <= '11111111111111111111111111111111' OR
									guid_field <= '3f3e3c3b-3a30-3938-3736-353433a2313e';

SELECT COUNT(*) FROM guid1 WHERE guid_field = '3f3e3c3b-3a30-3938-3736-353433a2313e' OR
							guid_field = '11111111111111111111111111111111';

INSERT INTO guid1(guid_field) VALUES('11111111-1111-1111-1111-111111111111');

SELECT count(*) FROM pg_class WHERE relkind='i' AND relname LIKE 'guid%';

INSERT INTO guid1(guid_field) VALUES('44444444-4444-4444-4444-444444444444');

INSERT INTO guid2(guid_field) VALUES('11111111-1111-1111-1111-111111111111');

INSERT INTO guid2(guid_field) VALUES('{22222222-2222-2222-2222-222222222222}');

INSERT INTO guid2(guid_field) VALUES('3f3e3c3b3a3039383736353433a2313e');

SELECT COUNT(*) FROM guid1 g1 INNER JOIN guid2 g2 ON g1.guid_field = g2.guid_field;

SELECT COUNT(*) FROM guid1 g1 LEFT JOIN guid2 g2 ON g1.guid_field = g2.guid_field WHERE g2.guid_field IS NULL;

TRUNCATE guid1;

INSERT INTO guid1 (guid_field) VALUES (gen_random_uuid());

INSERT INTO guid1 (guid_field) VALUES (gen_random_uuid());

SELECT count(DISTINCT guid_field) FROM guid1;

TRUNCATE guid1;

INSERT INTO guid1 (guid_field) VALUES (uuidv4());

INSERT INTO guid1 (guid_field) VALUES (uuidv4());

SELECT count(DISTINCT guid_field) FROM guid1;

TRUNCATE guid1;

INSERT INTO guid1 (guid_field) VALUES (uuidv7());

INSERT INTO guid1 (guid_field) VALUES (uuidv7());

INSERT INTO guid1 (guid_field) VALUES (uuidv7(INTERVAL '1 day'));

SELECT count(DISTINCT guid_field) FROM guid1;

INSERT INTO guid3 (guid_field) SELECT uuidv7() FROM generate_series(1, 10);

SELECT array_agg(id ORDER BY guid_field) FROM guid3;

WITH uuidts AS (
    SELECT y, ts as ts, lag(ts) OVER (ORDER BY y) AS prev_ts
    FROM (SELECT y, uuid_extract_timestamp(uuidv7((y || ' years')::interval)) AS ts
        FROM generate_series(1970 - extract(year from now())::int, 10888 - extract(year from now())::int) y)
)
SELECT y, ts, prev_ts FROM uuidts WHERE ts < prev_ts;

SELECT uuid_extract_version('11111111-1111-5111-8111-111111111111');

SELECT uuid_extract_version(gen_random_uuid());

SELECT uuid_extract_version('11111111-1111-1111-1111-111111111111');

SELECT uuid_extract_version(uuidv4());

SELECT uuid_extract_version(uuidv7());

SELECT uuid_extract_timestamp('C232AB00-9414-11EC-B3C8-9F6BDECED846') = 'Tuesday, February 22, 2022 2:22:22.00 PM GMT+05:00';

SELECT uuid_extract_timestamp('017F22E2-79B0-7CC3-98C4-DC0C0C07398F') = 'Tuesday, February 22, 2022 2:22:22.00 PM GMT+05:00';

SELECT uuid_extract_timestamp(gen_random_uuid());

SELECT uuid_extract_timestamp('11111111-1111-1111-1111-111111111111');

DROP TABLE guid1, guid2, guid3 CASCADE;
