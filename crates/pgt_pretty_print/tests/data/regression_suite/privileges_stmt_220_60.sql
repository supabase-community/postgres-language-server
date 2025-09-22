SELECT * FROM atest2 WHERE ( col1 IN ( SELECT b FROM atest1 ) );
