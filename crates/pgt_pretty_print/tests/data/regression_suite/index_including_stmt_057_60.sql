CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				EXCLUDE USING btree (c1 WITH =) INCLUDE(c3,c4));
