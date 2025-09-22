SELECT regexp_substr('1234567890', '(123)(4(56)(78))', 1, 1, 'i', 5) IS NULL AS t;
