SELECT explain_memoize('
SELECT * FROM expr_key t1 INNER JOIN expr_key t2
ON t1.x = t2.t::numeric AND t1.t::numeric = t2.x;', false);
