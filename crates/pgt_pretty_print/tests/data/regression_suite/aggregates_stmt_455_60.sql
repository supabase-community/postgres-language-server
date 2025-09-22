SELECT array_agg(c1 ORDER BY c2),c2
FROM agg_sort_order WHERE c2 < 100 GROUP BY c1 ORDER BY 2;
