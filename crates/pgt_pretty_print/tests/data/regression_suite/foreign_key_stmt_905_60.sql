CREATE TABLE ffk (a int, b int REFERENCES pk) PARTITION BY list (a);
