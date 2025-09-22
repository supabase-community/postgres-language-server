SELECT array_agg(x COLLATE "C" ORDER BY y COLLATE "POSIX") FROM collate_test10;
