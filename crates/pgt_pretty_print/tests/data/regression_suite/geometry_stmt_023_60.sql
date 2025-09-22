SELECT p1.f1, p2.f1, line(p1.f1, p2.f1)
  FROM POINT_TBL p1, POINT_TBL p2 WHERE p1.f1 <> p2.f1;
