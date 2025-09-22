SELECT 'a:1 b:3'::tsvector @@ 'a <3> b'::tsquery AS "false";
