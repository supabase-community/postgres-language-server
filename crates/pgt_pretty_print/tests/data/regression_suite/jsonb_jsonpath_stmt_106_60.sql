select * from jsonb_path_query('[1, "2", null]', '$[*] ? (@ != null)');
