CREATE TABLE fk_r_2 ( id int PRIMARY KEY, p_id int NOT NULL, p_jd int NOT NULL) PARTITION BY list (id)
