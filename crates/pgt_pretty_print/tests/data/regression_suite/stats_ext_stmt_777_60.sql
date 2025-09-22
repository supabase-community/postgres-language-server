SELECT statistics_name, most_common_vals FROM pg_stats_ext x
    WHERE tablename = 'stats_ext_tbl' ORDER BY ROW(x.*);
