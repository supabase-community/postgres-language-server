-- found by the fuzzer: the formatter formats a body with named parameters, like `:e`, which parses
-- with them converted: the program is the same. The comment holds a directive: --%c
create function d() returns int as $$select R[(C)],:e$$ language sql;
