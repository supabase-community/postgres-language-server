select jsonb_path_query('[1, 2, 3]', '($[*].a > 3).type()');
