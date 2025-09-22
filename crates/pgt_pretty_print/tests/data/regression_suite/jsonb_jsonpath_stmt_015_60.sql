select jsonb '{"a": {"b": 1}}' @? 'lax $.**{3}';
