SELECT jsonb_exists_any('{"a":null, "b":"qq"}', ARRAY['a','b']);
