-- found by the fuzzer: Go writes `%++` for `%%++`, one operator, so the text that is analysed
-- holds ` %++`, not `% ++`. The formatter can't put `%%` back into one operator, and the statement
-- stays as written
SELECT a%%++c;
-- `%%` before another character is formatted
select a %%    b;
