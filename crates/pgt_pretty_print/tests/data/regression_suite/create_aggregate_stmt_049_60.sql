CREATE OR REPLACE AGGREGATE sum3 (int8,int8,int8)
(
	stype = int8,
	sfunc = sum4
);
