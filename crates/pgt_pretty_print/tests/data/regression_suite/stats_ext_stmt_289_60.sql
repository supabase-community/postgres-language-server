SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE mod(a, 11) = 1 AND mod(b::int, 13) = 1');
