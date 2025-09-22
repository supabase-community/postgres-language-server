CREATE TABLE truncpart (a int REFERENCES truncprim)
  PARTITION BY RANGE (a);
