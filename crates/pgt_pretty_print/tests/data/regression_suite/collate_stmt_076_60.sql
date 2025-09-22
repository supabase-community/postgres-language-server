SELECT * FROM collate_test10 WHERE (x, y) NOT IN (SELECT y, x FROM collate_test10);
