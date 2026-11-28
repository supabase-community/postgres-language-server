-- found by the fuzzer: the formatter formats a body with named parameters, like `$abc`, which
-- parses with them converted: the program is the same
select '{}';
create function f() returns int language sql as 'select $abc';
