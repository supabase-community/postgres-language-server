insert into btree_tall_tbl select g, repeat('x', 250)
from generate_series(1, 130) g;
