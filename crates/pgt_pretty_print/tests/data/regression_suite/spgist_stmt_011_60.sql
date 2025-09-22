insert into spgist_box_tbl(b)
select box(point(i,j),point(i+s,j+s))
  from generate_series(1,100,5) i,
       generate_series(1,100,5) j,
       generate_series(1,10) s;
