select jsonb '{"a": 1, "b": [1, 2]}' @? 'lax $.keyvalue().key';
