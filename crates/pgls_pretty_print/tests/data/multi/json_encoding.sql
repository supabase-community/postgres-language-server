SELECT getdatabaseencoding() NOT IN ('UTF8', 'SQL_ASCII')
       AS skip_test ;

SELECT getdatabaseencoding();

SELECT '"\u"'::json;

SELECT '"\u00"'::json;

SELECT '"\u000g"'::json;

SELECT '"\u0000"'::json;

SELECT '"\uaBcD"'::json;

select json '{ "a":  "\ud83d\ude04\ud83d\udc36" }' -> 'a' as correct_in_utf8;

select json '{ "a":  "\ud83d\ud83d" }' -> 'a';

select json '{ "a":  "\ude04\ud83d" }' -> 'a';

select json '{ "a":  "\ud83dX" }' -> 'a';

select json '{ "a":  "\ude04X" }' -> 'a';

select json '{ "a":  "the Copyright \u00a9 sign" }' as correct_in_utf8;

select json '{ "a":  "dollar \u0024 character" }' as correct_everywhere;

select json '{ "a":  "dollar \\u0024 character" }' as not_an_escape;

select json '{ "a":  "null \u0000 escape" }' as not_unescaped;

select json '{ "a":  "null \\u0000 escape" }' as not_an_escape;

select json '{ "a":  "the Copyright \u00a9 sign" }' ->> 'a' as correct_in_utf8;

select json '{ "a":  "dollar \u0024 character" }' ->> 'a' as correct_everywhere;

select json '{ "a":  "dollar \\u0024 character" }' ->> 'a' as not_an_escape;

select json '{ "a":  "null \u0000 escape" }' ->> 'a' as fails;

select json '{ "a":  "null \\u0000 escape" }' ->> 'a' as not_an_escape;

SELECT '"\u"'::jsonb;

SELECT '"\u00"'::jsonb;

SELECT '"\u000g"'::jsonb;

SELECT '"\u0045"'::jsonb;

SELECT '"\u0000"'::jsonb;

SELECT octet_length('"\uaBcD"'::jsonb::text);

SELECT octet_length((jsonb '{ "a":  "\ud83d\ude04\ud83d\udc36" }' -> 'a')::text) AS correct_in_utf8;

SELECT jsonb '{ "a":  "\ud83d\ud83d" }' -> 'a';

SELECT jsonb '{ "a":  "\ude04\ud83d" }' -> 'a';

SELECT jsonb '{ "a":  "\ud83dX" }' -> 'a';

SELECT jsonb '{ "a":  "\ude04X" }' -> 'a';

SELECT jsonb '{ "a":  "the Copyright \u00a9 sign" }' as correct_in_utf8;

SELECT jsonb '{ "a":  "dollar \u0024 character" }' as correct_everywhere;

SELECT jsonb '{ "a":  "dollar \\u0024 character" }' as not_an_escape;

SELECT jsonb '{ "a":  "null \u0000 escape" }' as fails;

SELECT jsonb '{ "a":  "null \\u0000 escape" }' as not_an_escape;

SELECT jsonb '{ "a":  "the Copyright \u00a9 sign" }' ->> 'a' as correct_in_utf8;

SELECT jsonb '{ "a":  "dollar \u0024 character" }' ->> 'a' as correct_everywhere;

SELECT jsonb '{ "a":  "dollar \\u0024 character" }' ->> 'a' as not_an_escape;

SELECT jsonb '{ "a":  "null \u0000 escape" }' ->> 'a' as fails;

SELECT jsonb '{ "a":  "null \\u0000 escape" }' ->> 'a' as not_an_escape;

select * from pg_input_error_info('{ "a":  "\ud83d\ude04\ud83d\udc36" }', 'jsonb');
