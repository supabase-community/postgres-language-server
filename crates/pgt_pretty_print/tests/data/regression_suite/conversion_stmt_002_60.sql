CREATE FUNCTION test_enc_conversion(bytea, name, name, bool, validlen OUT int, result OUT bytea)
    AS 'regresslib', 'test_enc_conversion'
    LANGUAGE C STRICT;
