CREATE FUNCTION my_cmp (int4, int4)
RETURNS int LANGUAGE sql AS
$$
	SELECT
		CASE WHEN $1 < $2 THEN -1
				WHEN $1 > $2 THEN  1
				ELSE 0
		END;
$$;
