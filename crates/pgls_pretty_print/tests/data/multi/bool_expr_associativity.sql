CREATE TABLE demo_output AS
SELECT
	*
FROM
	demo
WHERE
	(
		demo.accounting_class ~ '^[0-9]+$'
		AND (
			demo.accounting_class >= '6000'
			AND demo.accounting_class <= '6629'
		)
		OR starts_with(demo.accounting_class, '71')
	)
	AND demo.changed_at > demo.created_at;

SELECT * FROM demo WHERE (a OR (b OR c)) AND d;

SELECT * FROM demo WHERE a AND (b OR c);
