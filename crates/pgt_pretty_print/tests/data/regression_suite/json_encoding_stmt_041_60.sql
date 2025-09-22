SELECT jsonb '{ "a":  "null \u0000 escape" }' ->> 'a' as fails;
