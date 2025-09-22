WITH RECURSIVE cte (a) as (
	SELECT a FROM duplicates
	UNION
	SELECT a FROM cte
)
SELECT a FROM cte;
