SELECT 'a:1 b:3'::tsvector @@ 'a <2> b'::tsquery AS "true";
