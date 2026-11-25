-- A language written as a string constant is kept whole; written as an identifier, the parser
-- would cut it down to 63 bytes.
CREATE FUNCTION f(int) RETURNS bool LANGUAGE 'sqlQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQ' AS 'SELECT $1 < 0';
