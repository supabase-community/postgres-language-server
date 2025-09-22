SELECT 'a:1 b:3'::tsvector @@ 'a <-> b'::tsquery AS "false";
