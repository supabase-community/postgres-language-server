select jsonb_set_lax('{"a":1,"b":2}', '{b}', null, null_value_treatment => 'use_json_null') as use_json_null;
