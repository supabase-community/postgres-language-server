SELECT jsonb '[{"a": 1}, {"a": 2}]' @@ '$[*].a > 1';
