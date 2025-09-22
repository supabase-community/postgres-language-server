SELECT *
FROM
	generate_series(1, 3) x,
	generate_series(1, 3) y,
	JSON_TABLE(jsonb
		'[[1,2,3],[2,3,4,5],[3,4,5,6]]',
		'strict $[*] ? (@[*] <= $x)'
		PASSING x AS x, y AS y
		COLUMNS (
			y text FORMAT JSON PATH '$',
			NESTED PATH 'strict $[*] ? (@ == $y)'
			COLUMNS (
				z int PATH '$'
			)
		)
	) jt;
