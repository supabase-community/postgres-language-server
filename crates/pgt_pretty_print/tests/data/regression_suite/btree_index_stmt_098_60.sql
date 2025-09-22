insert into btree_tall_tbl select g, NULL
from generate_series(50, 60) g;
