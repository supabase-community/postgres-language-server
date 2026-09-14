CREATE TABLE s.t AS
SELECT
	a.id,
	b.label
FROM s.a
	CROSS JOIN s.b;
