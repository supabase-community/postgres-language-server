insert into test_jsonb_subscript
  select s, ('{"' || s || '": "bar"}')::jsonb from repeat('xyzzy', 500) s;
