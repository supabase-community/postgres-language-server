insert into gist_tbl
  select box(point(0.05*i, 0.05*i)) from generate_series(0,10) as i;
