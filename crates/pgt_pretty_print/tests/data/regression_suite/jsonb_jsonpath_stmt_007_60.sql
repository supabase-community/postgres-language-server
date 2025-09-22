select jsonb '{"a": {"a": 12}}' @? '$.*.a';
