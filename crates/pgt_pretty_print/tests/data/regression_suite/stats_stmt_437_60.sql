CREATE TABLE brin_hot (
  id  integer PRIMARY KEY,
  val integer NOT NULL
) WITH (autovacuum_enabled = off, fillfactor = 70);
