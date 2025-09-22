SELECT 'a:1 b:3'::tsvector @@ 'a <0> a:*'::tsquery AS "true";
