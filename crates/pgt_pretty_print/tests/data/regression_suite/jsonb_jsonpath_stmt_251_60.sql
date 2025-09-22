select jsonb_path_query('[{},1]', '$[*].keyvalue()', silent => true);
