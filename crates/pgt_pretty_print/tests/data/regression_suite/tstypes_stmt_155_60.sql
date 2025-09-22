select to_tsvector('simple', 'x y q y') @@ '!(x <2> y)' AS "true";
