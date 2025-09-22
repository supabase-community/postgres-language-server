SELECT double_append(array_append(ARRAY[q1], q2), q3)
  FROM (VALUES(1,2,3), (4,5,6)) v(q1,q2,q3);
