CREATE FUNCTION interpt_pp(path, path)
    RETURNS point
    AS 'regresslib'
    LANGUAGE C STRICT;
