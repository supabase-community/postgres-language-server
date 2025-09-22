select jsonb '{"a": 1, "b":1}' @? '$ ? (@.a == @.b)';
