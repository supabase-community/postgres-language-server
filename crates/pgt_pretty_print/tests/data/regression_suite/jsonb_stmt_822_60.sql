select jsonb_delete_path('{"a":[]}', '{"a",-2147483648}');
