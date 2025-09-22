INSERT INTO ab1
SELECT x / 10, x / 3,
    '2020-10-01'::timestamp + x * interval '1 day',
    '2020-10-01'::timestamptz + x * interval '1 day'
FROM generate_series(1, 100) x;
