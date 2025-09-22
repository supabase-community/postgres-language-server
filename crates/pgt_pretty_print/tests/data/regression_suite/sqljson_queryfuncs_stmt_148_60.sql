SELECT JSON_QUERY(jsonb '[1,2]', '$[*]' DEFAULT '"empty"' ON ERROR);
