SELECT r, count(*)
FROM (SELECT random() r FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;

SELECT count(*) FILTER (WHERE r < 0 OR r >= 1) AS out_of_range,
       (count(*) FILTER (WHERE r < 0.01)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 0.99)) > 0 AS has_large
FROM (SELECT random() r FROM generate_series(1, 2000)) ss;

CREATE FUNCTION ks_test_uniform_random()
RETURNS boolean AS
$$
DECLARE
  n int := 1000;        -- Number of samples
  c float8 := 1.94947;  -- Critical value for 99.9% confidence
  ok boolean;
BEGIN
  ok := (
    WITH samples AS (
      SELECT random() r FROM generate_series(1, n) ORDER BY 1
    ), indexed_samples AS (
      SELECT (row_number() OVER())-1.0 i, r FROM samples
    )
    SELECT max(abs(i/n-r)) < c / sqrt(n) FROM indexed_samples
  );
  RETURN ok;
END
$$
LANGUAGE plpgsql;

SELECT ks_test_uniform_random() OR
       ks_test_uniform_random() OR
       ks_test_uniform_random() AS uniform;

SELECT r, count(*)
FROM (SELECT random_normal() r FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;

SELECT r, count(*)
FROM (SELECT random_normal(10, 0) r FROM generate_series(1, 100)) ss
GROUP BY r;

SELECT r, count(*)
FROM (SELECT random_normal(-10, 0) r FROM generate_series(1, 100)) ss
GROUP BY r;

CREATE FUNCTION ks_test_normal_random()
RETURNS boolean AS
$$
DECLARE
  n int := 1000;        -- Number of samples
  c float8 := 1.94947;  -- Critical value for 99.9% confidence
  ok boolean;
BEGIN
  ok := (
    WITH samples AS (
      SELECT random_normal() r FROM generate_series(1, n) ORDER BY 1
    ), indexed_samples AS (
      SELECT (row_number() OVER())-1.0 i, r FROM samples
    )
    SELECT max(abs((1+erf(r/sqrt(2)))/2 - i/n)) < c / sqrt(n)
    FROM indexed_samples
  );
  RETURN ok;
END
$$
LANGUAGE plpgsql;

SELECT ks_test_normal_random() OR
       ks_test_normal_random() OR
       ks_test_normal_random() AS standard_normal;

SELECT random(1, 0);

SELECT random(1000000000001, 1000000000000);

SELECT random(-2.0, -3.0);

SELECT random('NaN'::numeric, 10);

SELECT random('-Inf'::numeric, 0);

SELECT random(0, 'NaN'::numeric);

SELECT random(0, 'Inf'::numeric);

SELECT random(101, 101);

SELECT random(1000000000001, 1000000000001);

SELECT random(3.14, 3.14);

SELECT r, count(*)
FROM (SELECT random(-2147483648, 2147483647) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 2;

SELECT r, count(*)
FROM (SELECT random_normal(-9223372036854775808, 9223372036854775807) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;

SELECT r, count(*)
FROM (SELECT random_normal(0, 1 - 1e-15) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;

SELECT (count(*) FILTER (WHERE r < -2104533975)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 2104533974)) > 0 AS has_large
FROM (SELECT random(-2147483648, 2147483647) r FROM generate_series(1, 2000)) ss;

SELECT count(*) FILTER (WHERE r < -1500000000 OR r > 1500000000) AS out_of_range,
       (count(*) FILTER (WHERE r < -1470000000)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 1470000000)) > 0 AS has_large
FROM (SELECT random(-1500000000, 1500000000) r FROM generate_series(1, 2000)) ss;

SELECT (count(*) FILTER (WHERE r < -9038904596117680292)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 9038904596117680291)) > 0 AS has_large
FROM (SELECT random(-9223372036854775808, 9223372036854775807) r
      FROM generate_series(1, 2000)) ss;

SELECT count(*) FILTER (WHERE r < -1500000000000000 OR r > 1500000000000000) AS out_of_range,
       (count(*) FILTER (WHERE r < -1470000000000000)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 1470000000000000)) > 0 AS has_large
FROM (SELECT random(-1500000000000000, 1500000000000000) r
      FROM generate_series(1, 2000)) ss;

SELECT count(*) FILTER (WHERE r < -1.5 OR r > 1.5) AS out_of_range,
       (count(*) FILTER (WHERE r < -1.47)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 1.47)) > 0 AS has_large
FROM (SELECT random(-1.500000000000000, 1.500000000000000) r
      FROM generate_series(1, 2000)) ss;

SELECT min(r), max(r), count(r) FROM (
  SELECT DISTINCT random(-50, 49) r FROM generate_series(1, 2500));

SELECT min(r), max(r), count(r) FROM (
  SELECT DISTINCT random(123000000000, 123000000099) r
  FROM generate_series(1, 2500));

SELECT min(r), max(r), count(r) FROM (
  SELECT DISTINCT random(-0.5, 0.49) r FROM generate_series(1, 2500));

CREATE FUNCTION ks_test_uniform_random_int_in_range()
RETURNS boolean AS
$$
DECLARE
  n int := 1000;        -- Number of samples
  c float8 := 1.94947;  -- Critical value for 99.9% confidence
  ok boolean;
BEGIN
  ok := (
    WITH samples AS (
      SELECT random(0, 999999) / 1000000.0 r FROM generate_series(1, n) ORDER BY 1
    ), indexed_samples AS (
      SELECT (row_number() OVER())-1.0 i, r FROM samples
    )
    SELECT max(abs(i/n-r)) < c / sqrt(n) FROM indexed_samples
  );
  RETURN ok;
END
$$
LANGUAGE plpgsql;

SELECT ks_test_uniform_random_int_in_range() OR
       ks_test_uniform_random_int_in_range() OR
       ks_test_uniform_random_int_in_range() AS uniform_int;

CREATE FUNCTION ks_test_uniform_random_bigint_in_range()
RETURNS boolean AS
$$
DECLARE
  n int := 1000;        -- Number of samples
  c float8 := 1.94947;  -- Critical value for 99.9% confidence
  ok boolean;
BEGIN
  ok := (
    WITH samples AS (
      SELECT random(0, 999999999999) / 1000000000000.0 r FROM generate_series(1, n) ORDER BY 1
    ), indexed_samples AS (
      SELECT (row_number() OVER())-1.0 i, r FROM samples
    )
    SELECT max(abs(i/n-r)) < c / sqrt(n) FROM indexed_samples
  );
  RETURN ok;
END
$$
LANGUAGE plpgsql;

SELECT ks_test_uniform_random_bigint_in_range() OR
       ks_test_uniform_random_bigint_in_range() OR
       ks_test_uniform_random_bigint_in_range() AS uniform_bigint;

CREATE FUNCTION ks_test_uniform_random_numeric_in_range()
RETURNS boolean AS
$$
DECLARE
  n int := 1000;        -- Number of samples
  c float8 := 1.94947;  -- Critical value for 99.9% confidence
  ok boolean;
BEGIN
  ok := (
    WITH samples AS (
      SELECT random(0, 0.999999) r FROM generate_series(1, n) ORDER BY 1
    ), indexed_samples AS (
      SELECT (row_number() OVER())-1.0 i, r FROM samples
    )
    SELECT max(abs(i/n-r)) < c / sqrt(n) FROM indexed_samples
  );
  RETURN ok;
END
$$
LANGUAGE plpgsql;

SELECT ks_test_uniform_random_numeric_in_range() OR
       ks_test_uniform_random_numeric_in_range() OR
       ks_test_uniform_random_numeric_in_range() AS uniform_numeric;

SELECT setseed(0.5);

SELECT random() FROM generate_series(1, 10);

SET extra_float_digits = -1;

SELECT random_normal() FROM generate_series(1, 10);

SELECT random_normal(mean => 1, stddev => 0.1) r FROM generate_series(1, 10);

SELECT random(1, 6) FROM generate_series(1, 10);

SELECT random(-2147483648, 2147483647) FROM generate_series(1, 10);

SELECT random(-9223372036854775808, 9223372036854775807) FROM generate_series(1, 10);

SELECT random(-1e30, 1e30) FROM generate_series(1, 10);

SELECT random(-0.4, 0.4) FROM generate_series(1, 10);

SELECT random(0, 1 - 1e-30) FROM generate_series(1, 10);

SELECT n, random(0, trim_scale(abs(1 - 10.0^(-n)))) FROM generate_series(-20, 20) n;

SELECT random('1979-02-08'::date,'2025-07-03'::date) AS random_date_multiple_years;

SELECT random('4714-11-24 BC'::date,'5874897-12-31 AD'::date) AS random_date_maximum_range;

SELECT random('1979-02-08'::date,'1979-02-08'::date) AS random_date_empty_range;

SELECT random('2024-12-31'::date, '2024-01-01'::date);

SELECT random('-infinity'::date, '2024-01-01'::date);

SELECT random('2024-12-31'::date, 'infinity'::date);

SELECT random('1979-02-08'::timestamp,'2025-07-03'::timestamp) AS random_timestamp_multiple_years;

SELECT random('4714-11-24 BC'::timestamp,'294276-12-31 23:59:59.999999'::timestamp) AS random_timestamp_maximum_range;

SELECT random('2024-07-01 12:00:00.000001'::timestamp, '2024-07-01 12:00:00.999999'::timestamp) AS random_narrow_range;

SELECT random('1979-02-08'::timestamp,'1979-02-08'::timestamp) AS random_timestamp_empty_range;

SELECT random('2024-12-31'::timestamp, '2024-01-01'::timestamp);

SELECT random('-infinity'::timestamp, '2024-01-01'::timestamp);

SELECT random('2024-12-31'::timestamp, 'infinity'::timestamp);

SELECT random('1979-02-08 +01'::timestamptz,'2025-07-03 +02'::timestamptz) AS random_timestamptz_multiple_years;

SELECT random('4714-11-24 BC +00'::timestamptz,'294276-12-31 23:59:59.999999 +00'::timestamptz) AS random_timestamptz_maximum_range;

SELECT random('2024-07-01 12:00:00.000001 +04'::timestamptz, '2024-07-01 12:00:00.999999 +04'::timestamptz) AS random_timestamptz_narrow_range;

SELECT random('1979-02-08 +05'::timestamptz,'1979-02-08 +05'::timestamptz) AS random_timestamptz_empty_range;

SELECT random('2024-01-01 +06'::timestamptz, '2024-01-01 +07'::timestamptz);

SELECT random('-infinity'::timestamptz, '2024-01-01 +07'::timestamptz);

SELECT random('2024-01-01 +06'::timestamptz, 'infinity'::timestamptz);
