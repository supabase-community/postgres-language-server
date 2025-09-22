SELECT jsonb_path_match('[{"a": 1}, {"a": 2}]', '$[*].a > 1');
