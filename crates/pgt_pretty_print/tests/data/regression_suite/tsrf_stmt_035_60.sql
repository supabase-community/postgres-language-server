SELECT few.dataa, count(*), min(id), max(id), generate_series(1,3) FROM few GROUP BY few.dataa ORDER BY 5, 1;
