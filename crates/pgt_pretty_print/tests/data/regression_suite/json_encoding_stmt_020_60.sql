select json '{ "a":  "null \u0000 escape" }' ->> 'a' as fails;
