CREATE STATISTICS expr_stats_1 (mcv) ON a, b, (b || c), (c || b) FROM expr_stats;
