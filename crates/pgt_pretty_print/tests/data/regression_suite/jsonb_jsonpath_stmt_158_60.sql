select jsonb '{"c": {"a": 1, "b":1}}' @? '$ ? (@.a == @.b)';
