select jsonb_path_query('[1, 2, 3]', 'strict ($[*].a > 3).type()');
