SELECT (r).column2 AS col_a, (ss.a).x, (c).*, (k).word FROM cte AS t WHERE (r).column1 = (rr).column1 AND (SELECT (c1).f1 > 0);
