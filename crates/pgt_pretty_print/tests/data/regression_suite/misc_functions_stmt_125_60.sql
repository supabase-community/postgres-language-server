SELECT setting::int8 AS segment_size
FROM pg_settings
WHERE name = 'wal_segment_size'
