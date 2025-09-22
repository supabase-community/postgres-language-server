create index spgist_point_idx on spgist_point_tbl using spgist(p) with (fillfactor = 75);
