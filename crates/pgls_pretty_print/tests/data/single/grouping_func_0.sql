SELECT category, GROUPING(category) FROM products GROUP BY ROLLUP (category);
