SELECT percentile_cont(0.75) WITHIN GROUP (ORDER BY salary DESC) FILTER (WHERE salary IS NOT NULL)
FROM employees;
