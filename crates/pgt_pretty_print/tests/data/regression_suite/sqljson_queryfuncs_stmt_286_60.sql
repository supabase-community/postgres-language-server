SELECT JSON_QUERY(jsonb 'null', '$"Xyz"' PASSING 1 AS "Xyz");
