SELECT JSON_QUERY(jsonb '[]', '$[*]' DEFAULT '"empty"' ON EMPTY);
