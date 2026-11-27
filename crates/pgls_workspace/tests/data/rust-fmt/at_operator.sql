-- found by the fuzzer: an `@` after an operand is an operator, not the start of an `@name`
-- parameter, which the formatter wrote into the file as `"jsonb_$1"`
SELECT jsonb_@path_matci('{}', '$', silent => false);
select a @b,   (a) @b from t where x = {};
