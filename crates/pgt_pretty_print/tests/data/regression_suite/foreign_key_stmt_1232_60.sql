CREATE TABLE fk_r   ( id int PRIMARY KEY, p_id int NOT NULL, p_jd int NOT NULL,
       FOREIGN KEY (p_id, p_jd) REFERENCES fk_p (id, jd)
  ) PARTITION BY list (id);
