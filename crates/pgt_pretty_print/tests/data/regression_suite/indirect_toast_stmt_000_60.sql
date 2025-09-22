CREATE FUNCTION make_tuple_indirect (record)
        RETURNS record
        AS 'regresslib'
        LANGUAGE C STRICT;
