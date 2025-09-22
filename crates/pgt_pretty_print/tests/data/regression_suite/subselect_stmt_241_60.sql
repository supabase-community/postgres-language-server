select * from
(values
  (3 not in (select * from (values (1), (2)) ss1)),
  (false)
) ss;
