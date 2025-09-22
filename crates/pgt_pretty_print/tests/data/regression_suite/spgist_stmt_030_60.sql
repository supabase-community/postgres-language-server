insert into spgist_unlogged_tbl(b)
select box(point(i,j))
  from generate_series(1,100,5) i,
       generate_series(1,10,5) j;
