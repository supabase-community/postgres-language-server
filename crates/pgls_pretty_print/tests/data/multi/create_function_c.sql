LOAD 'regresslib';

CREATE FUNCTION test1 (int) RETURNS int LANGUAGE C
    AS 'nosuchfile';

CREATE FUNCTION test1 (int) RETURNS int LANGUAGE C
    AS 'regresslib', 'nosuchsymbol';

SELECT regexp_replace('LAST_ERROR_MESSAGE', 'file ".*"', 'file "..."');

CREATE FUNCTION test1 (int) RETURNS int LANGUAGE internal
    AS 'nosuch';
