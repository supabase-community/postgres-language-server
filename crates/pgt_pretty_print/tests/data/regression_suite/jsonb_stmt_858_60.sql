select jsonb_set_lax('{"a":1,"b":2}', '{b}', null, true, 'no_such_treatment');
