select jsonb_path_query('[1, 2, 3]', '($[*] > 3).type()');
