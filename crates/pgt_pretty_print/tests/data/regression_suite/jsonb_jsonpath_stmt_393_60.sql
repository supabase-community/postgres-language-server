select jsonb_path_query_array('[1, "yes", false]', '$[*].boolean()');
