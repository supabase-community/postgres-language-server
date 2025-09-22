SELECT JSON_QUERY(jsonb '"aaa"', '$.a' RETURNING char(2) OMIT QUOTES DEFAULT '"bb"'::jsonb ON EMPTY);
