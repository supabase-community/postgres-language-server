SELECT jsonb '{ "a":  "dollar \u0024 character" }' ->> 'a' as correct_everywhere;
