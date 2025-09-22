CREATE FUNCTION is_catalog_text_unique_index_oid(oid) RETURNS bool
    AS 'regresslib', 'is_catalog_text_unique_index_oid'
    LANGUAGE C STRICT;
