select * from nestjsonb where j @> '{"a":[[{"x":2}]]}'::jsonb;
