SELECT percentile_disc(0.5) WITHIN GROUP (ORDER BY score) FILTER (WHERE score > 0)
FROM (VALUES (1), (2), (3)) AS scores(score);

SELECT percentile_cont(0.9) WITHIN GROUP (ORDER BY duration) FILTER (WHERE duration IS NOT NULL)
FROM (VALUES (INTERVAL '1 hour'), (INTERVAL '2 hours')) AS durations(duration);
