select jsonb_path_query_array('[1.23, "yes", false]', '$[*].string()');
