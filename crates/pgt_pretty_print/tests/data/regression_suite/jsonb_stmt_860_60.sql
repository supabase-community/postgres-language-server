select jsonb_set_lax('{"a":1,"b":2}', '{b}', null, null_value_treatment => 'return_target') as return_target;
