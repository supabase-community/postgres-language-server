select to_tsvector('simple', 'x y z') @@ '(x | y <-> z) <-> q' AS "false";
