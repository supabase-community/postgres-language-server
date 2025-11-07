SELECT getdatabaseencoding() NOT IN ('UTF8', 'SQL_ASCII')
       AS skip_test ;

SELECT getdatabaseencoding();

SELECT '"\u"'::jsonpath;

SELECT '"\u00"'::jsonpath;

SELECT '"\u000g"'::jsonpath;

SELECT '"\u0000"'::jsonpath;

SELECT '"\uaBcD"'::jsonpath;

select '"\ud83d\ude04\ud83d\udc36"'::jsonpath as correct_in_utf8;

select '"\ud83d\ud83d"'::jsonpath;

select '"\ude04\ud83d"'::jsonpath;

select '"\ud83dX"'::jsonpath;

select '"\ude04X"'::jsonpath;

select '"the Copyright \u00a9 sign"'::jsonpath as correct_in_utf8;

select '"dollar \u0024 character"'::jsonpath as correct_everywhere;

select '"dollar \\u0024 character"'::jsonpath as not_an_escape;

select '"null \u0000 escape"'::jsonpath as not_unescaped;

select '"null \\u0000 escape"'::jsonpath as not_an_escape;

SELECT '$."\u"'::jsonpath;

SELECT '$."\u00"'::jsonpath;

SELECT '$."\u000g"'::jsonpath;

SELECT '$."\u0000"'::jsonpath;

SELECT '$."\uaBcD"'::jsonpath;

select '$."\ud83d\ude04\ud83d\udc36"'::jsonpath as correct_in_utf8;

select '$."\ud83d\ud83d"'::jsonpath;

select '$."\ude04\ud83d"'::jsonpath;

select '$."\ud83dX"'::jsonpath;

select '$."\ude04X"'::jsonpath;

select '$."the Copyright \u00a9 sign"'::jsonpath as correct_in_utf8;

select '$."dollar \u0024 character"'::jsonpath as correct_everywhere;

select '$."dollar \\u0024 character"'::jsonpath as not_an_escape;

select '$."null \u0000 escape"'::jsonpath as not_unescaped;

select '$."null \\u0000 escape"'::jsonpath as not_an_escape;
