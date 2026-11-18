WITH RECURSIVE g(a) AS (SELECT 1) CYCLE a SET c TO point '(1,1)' DEFAULT interval '1' hour USING p SELECT * FROM g;
