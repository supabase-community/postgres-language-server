CREATE TABLE gtestm (
  id int PRIMARY KEY,
  f1 int,
  f2 int,
  f3 int GENERATED ALWAYS AS (f1 * 2) STORED,
  f4 int GENERATED ALWAYS AS (f2 * 2) STORED
);
