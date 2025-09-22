SELECT count(*)
  FROM btg t1 JOIN btg t2 ON t1.w = t2.w AND t1.x = t2.x AND t1.z = t2.z
  GROUP BY t1.w, t1.z, t1.x;
