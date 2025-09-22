SELECT jsonb '{ "a":  "null \\u0000 escape" }' ->> 'a' as not_an_escape;
