select 'foo'::text = any((select array['abc','def','foo']::text[])::text[]);
