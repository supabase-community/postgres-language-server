CREATE TABLE parted_si (
  id int not null,
  data text not null,
  -- prevent use of bulk insert by having a volatile function
  rand float8 not null default random()
)
PARTITION BY LIST((id % 2));
