SELECT c AS cidr, abbrev(c) AS "abbrev(cidr)",
  i AS inet, abbrev(i) AS "abbrev(inet)" FROM INET_TBL;
