SELECT regexp_substr('foo', 'foo(bar)?', 1, 1, '', 1) IS NULL AS t;
