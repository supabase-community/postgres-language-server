CREATE OR REPLACE AGGREGATE myavg (order by numeric)
(
	stype = numeric,
	sfunc = numeric_add
);
