-- found by the fuzzer: `@@g` is the operator `@@` and `g` as written, not `@` and the parameter `@g`, also
-- in a statement with a directive: the statement stays as written
select#j@@g%(%g|e);
select   1;
