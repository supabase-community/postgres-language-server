SELECT * FROM atest12v x, atest12v y
  WHERE x.a = y.b and abs(y.a) <<< 5;
