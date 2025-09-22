SELECT * FROM collate_test10 WHERE (x, y) NOT IN (SELECT y COLLATE "C", x COLLATE "POSIX" FROM collate_test10);
