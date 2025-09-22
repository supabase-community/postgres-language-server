CREATE STATISTICS expr_stats_1 (mcv) ON a, b, (2*a), (3*b), (a+b), (a-b) FROM expr_stats;
