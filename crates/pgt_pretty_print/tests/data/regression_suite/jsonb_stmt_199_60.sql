SELECT jsonb_exists_any('{"a":null, "b":"qq"}', ARRAY['b','a']);
