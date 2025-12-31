CREATE TABLE pred_tab (a int NOT NULL, b int, c int NOT NULL);

SELECT * FROM pred_tab t WHERE t.a IS NOT NULL;

SELECT * FROM pred_tab t WHERE t.b IS NOT NULL;

SELECT * FROM pred_tab t WHERE t.a IS NULL;

SELECT * FROM pred_tab t WHERE t.b IS NULL;

SELECT * FROM pred_tab t WHERE t.a IS NOT NULL OR t.b = 1;

SELECT * FROM pred_tab t WHERE t.b IS NOT NULL OR t.a = 1;

SELECT * FROM pred_tab t WHERE t.a IS NULL OR t.c IS NULL;

SELECT * FROM pred_tab t WHERE t.b IS NULL OR t.c IS NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a IS NOT NULL;

SELECT * FROM pred_tab t1
    FULL JOIN pred_tab t2 ON t1.a = t2.a
    LEFT JOIN pred_tab t3 ON t2.a IS NOT NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a IS NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a = 1
    LEFT JOIN pred_tab t3 ON t2.a IS NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a IS NOT NULL OR t2.b = 1;

SELECT * FROM pred_tab t1
    FULL JOIN pred_tab t2 ON t1.a = t2.a
    LEFT JOIN pred_tab t3 ON t2.a IS NOT NULL OR t2.b = 1;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON (t1.a IS NULL OR t1.c IS NULL);

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a = 1
    LEFT JOIN pred_tab t3 ON t2.a IS NULL OR t2.c IS NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON EXISTS
        (SELECT 1 FROM pred_tab t3, pred_tab t4, pred_tab t5, pred_tab t6
         WHERE t1.a = t3.a AND t6.a IS NOT NULL);

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON EXISTS
        (SELECT 1 FROM pred_tab t3, pred_tab t4, pred_tab t5, pred_tab t6
         WHERE t1.a = t3.a AND t6.a IS NULL);

DROP TABLE pred_tab;

CREATE TABLE pred_parent (a int);

CREATE TABLE pred_child () INHERITS (pred_parent);

ALTER TABLE ONLY pred_parent ALTER a SET NOT NULL;

SELECT * FROM pred_parent WHERE a IS NOT NULL;

SELECT * FROM pred_parent WHERE a IS NULL;

ALTER TABLE pred_parent ALTER a DROP NOT NULL;

ALTER TABLE pred_child ALTER a SET NOT NULL;

SELECT * FROM pred_parent WHERE a IS NOT NULL;

SELECT * FROM pred_parent WHERE a IS NULL;

DROP TABLE pred_parent, pred_child;

CREATE TABLE pred_tab (a int, b int);

CREATE TABLE pred_tab_notnull (a int, b int NOT NULL);

INSERT INTO pred_tab VALUES (1, 1);

INSERT INTO pred_tab VALUES (2, 2);

INSERT INTO pred_tab_notnull VALUES (2, 2);

INSERT INTO pred_tab_notnull VALUES (3, 3);

ANALYZE pred_tab;

ANALYZE pred_tab_notnull;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON TRUE
    LEFT JOIN pred_tab_notnull t3 ON t2.a = t3.a
    LEFT JOIN pred_tab t4 ON t3.b IS NOT NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON TRUE
    LEFT JOIN pred_tab_notnull t3 ON t2.a = t3.a
    LEFT JOIN pred_tab t4 ON t3.b IS NOT NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON TRUE
    LEFT JOIN pred_tab_notnull t3 ON t2.a = t3.a
    LEFT JOIN pred_tab t4 ON t3.b IS NULL AND t3.a IS NOT NULL;

SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON TRUE
    LEFT JOIN pred_tab_notnull t3 ON t2.a = t3.a
    LEFT JOIN pred_tab t4 ON t3.b IS NULL AND t3.a IS NOT NULL;

DROP TABLE pred_tab;

DROP TABLE pred_tab_notnull;

CREATE TABLE pred_tab1 (a int NOT NULL, b int,
	CONSTRAINT check_tab1 CHECK (a IS NULL OR b > 2));

CREATE TABLE pred_tab2 (a int, b int,
	CONSTRAINT check_a CHECK (a IS NOT NULL));

SET constraint_exclusion TO ON;

SELECT * FROM pred_tab1, pred_tab2 WHERE pred_tab2.a IS NULL;

SELECT * FROM pred_tab2, pred_tab1 WHERE pred_tab1.a IS NULL OR pred_tab1.b < 2;

RESET constraint_exclusion;

DROP TABLE pred_tab1;

DROP TABLE pred_tab2;
