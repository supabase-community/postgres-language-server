-- found by the fuzzer: the directive `%c` in the parameter `@mpo%ca` stays as written, and its `%`
-- isn't the `%` of `%%`: the statement stays as written
SELECT
i@mpo%ca;
select   1;
