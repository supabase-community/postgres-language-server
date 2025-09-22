CREATE FUNCTION errfunc() RETURNS int LANGUAGE SQL AS 'SELECT 1'
SET transaction_read_only = on;
