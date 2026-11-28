-- found by the fuzzer: a quoted name with only escapes, like `"}}"`, is a name, also as an alias:
-- the statement stays as written
SELECT S "}}";
select a   "{{x}}" from t where b = {};
