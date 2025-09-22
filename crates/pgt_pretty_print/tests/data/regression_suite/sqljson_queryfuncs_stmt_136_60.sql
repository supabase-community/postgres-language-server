SELECT JSON_QUERY(jsonb '[]', '$[*]' NULL ON EMPTY);
