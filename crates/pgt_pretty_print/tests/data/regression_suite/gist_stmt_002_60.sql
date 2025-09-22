create index gist_pointidx2 on gist_point_tbl using gist(p) with (buffering = on, fillfactor=50);
