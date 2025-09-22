SELECT JSON_VALUE(jsonb '"\"aaa\""', '$' RETURNING jsonb);
