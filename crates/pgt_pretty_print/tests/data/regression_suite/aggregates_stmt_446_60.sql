SELECT c1.y,c1.x FROM group_agg_pk c1
  JOIN group_agg_pk c2
  ON c1.x = c2.x
GROUP BY c1.y,c1.x,c2.x;
