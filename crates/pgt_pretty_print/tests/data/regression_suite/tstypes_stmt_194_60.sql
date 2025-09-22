SELECT 'a:1 b:2'::tsvector @@ 'a <1> b'::tsquery AS "true";
