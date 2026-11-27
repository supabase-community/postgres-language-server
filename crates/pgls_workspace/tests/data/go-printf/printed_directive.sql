-- found by the fuzzer: `%a` is no directive, but formatted, `timest%amptz '...'` is
-- `timest % cast('...' as amptz)`, whose `% c` is one: the statement stays as written
select timest%amptz '1972-10-10 03:00' from t where x = '%s';
select   1;
