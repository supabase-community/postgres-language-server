SELECT c AS cidr, set_masklen(cidr(text(c)), -1) AS "set_masklen(cidr)",
  i AS inet, set_masklen(inet(text(i)), -1) AS "set_masklen(inet)" FROM INET_TBL;
