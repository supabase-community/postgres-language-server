select to_tsvector('simple', 'x y q') @@ '(x | !!z) <-> y <-> q' AS "true";
