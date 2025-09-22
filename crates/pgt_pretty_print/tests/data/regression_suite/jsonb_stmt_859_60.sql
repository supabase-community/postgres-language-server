select jsonb_set_lax('{"a":1,"b":2}', '{b}', null, null_value_treatment => 'raise_exception') as raise_exception;
