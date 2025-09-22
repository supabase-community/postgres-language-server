select jsonb '{"a": {"b": 1}}' @? '$.**.b ? ( @ > 0)';
