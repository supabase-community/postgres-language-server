SELECT * FROM few f1,
  (SELECT unnest(ARRAY[1,2]) FROM few f2 WHERE false OFFSET 0) ss;
