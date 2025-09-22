CREATE STATISTICS s_expr ON mod(id, 2), lower(col) FROM stats_ext_tbl;
