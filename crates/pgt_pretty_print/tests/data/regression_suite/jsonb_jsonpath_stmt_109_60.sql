select * from jsonb_path_query('[]', 'strict $ ? (@ == @)');
