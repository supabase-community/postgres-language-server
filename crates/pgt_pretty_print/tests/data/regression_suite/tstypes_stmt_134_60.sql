select to_tsvector('simple', 'q x') @@ '(x | y <-> z) <-> q' AS "false";
