INSERT INTO brin_timestamp_test
SELECT i FROM generate_series('2000-01-01'::timestamp, '2000-02-09'::timestamp, '1 day'::interval) s(i);
