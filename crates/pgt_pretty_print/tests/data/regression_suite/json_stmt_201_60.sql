select '{"a": [{"b": "c"}, {"b": "cc"}]}'::json #>> array['a','z','b'];
