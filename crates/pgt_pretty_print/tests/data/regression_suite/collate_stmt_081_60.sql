CREATE FUNCTION vc (text) RETURNS text LANGUAGE sql
    AS 'select $1::varchar';
