select to_tsvector('simple', 'x y q') @@ '(!x | !y) <-> y <-> q' AS "true";
