CREATE TABLE pred_tab1 (a int NOT NULL, b int,
	CONSTRAINT check_tab1 CHECK (a IS NULL OR b > 2));
