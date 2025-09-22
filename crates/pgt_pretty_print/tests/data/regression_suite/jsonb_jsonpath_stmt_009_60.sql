select jsonb '{"b": {"a": 12}}' @? '$.*.b';
