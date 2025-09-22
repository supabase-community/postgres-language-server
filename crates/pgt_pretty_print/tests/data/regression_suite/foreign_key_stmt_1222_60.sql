CREATE TABLE fk_p ( id int, jd int, PRIMARY KEY(id, jd)) PARTITION BY list (id)
