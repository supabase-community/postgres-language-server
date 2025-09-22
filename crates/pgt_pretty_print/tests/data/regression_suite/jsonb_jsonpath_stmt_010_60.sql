select jsonb '{"b": {"a": 12}}' @? 'strict $.*.b';
