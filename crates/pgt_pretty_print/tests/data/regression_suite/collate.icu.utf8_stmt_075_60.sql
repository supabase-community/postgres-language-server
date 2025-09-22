CREATE VIEW collview3 AS SELECT a, lower((x || x) COLLATE "C") FROM collate_test10;
