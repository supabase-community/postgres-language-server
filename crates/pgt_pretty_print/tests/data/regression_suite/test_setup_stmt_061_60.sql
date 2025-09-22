CREATE FUNCTION binary_coercible(oid, oid)
    RETURNS bool
    AS 'regresslib', 'binary_coercible'
    LANGUAGE C STRICT STABLE PARALLEL SAFE;
