SELECT i as pow,
	round((-2.5 * 10 ^ i)::numeric, -i),
	round((-1.5 * 10 ^ i)::numeric, -i),
	round((-0.5 * 10 ^ i)::numeric, -i),
	round((0.5 * 10 ^ i)::numeric, -i),
	round((1.5 * 10 ^ i)::numeric, -i),
	round((2.5 * 10 ^ i)::numeric, -i)
FROM generate_series(-5,5) AS t(i);
