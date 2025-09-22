SELECT JSON_ARRAYAGG(i::text::json)
FROM generate_series(1, 5) i;
