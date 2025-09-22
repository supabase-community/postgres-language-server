SELECT $$
    SELECT col12, count(distinct a.col1), count(distinct a.col2), count(distinct b.col1), count(distinct b.col2), count(*)
    FROM test_mark_restore a
        JOIN test_mark_restore b USING(col12)
    GROUP BY 1
    HAVING count(*) > 1
    ORDER BY 2 DESC, 1 DESC, 3 DESC, 4 DESC, 5 DESC, 6 DESC
    LIMIT 10
$$ AS qry ;
