SELECT * FROM check_estimated_rows('SELECT a + 1, b FROM ONLY stxdinp GROUP BY 1, 2');
