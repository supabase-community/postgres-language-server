select jsonb_path_query('[]', '$[last ? (exists(last))]');
