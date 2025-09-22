SELECT c.oid, c.conname
FROM pg_conversion as c
WHERE c.conproc = 0 OR
    pg_encoding_to_char(conforencoding) = '' OR
    pg_encoding_to_char(contoencoding) = '';
