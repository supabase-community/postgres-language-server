SELECT c AS cidr, broadcast(c) AS "broadcast(cidr)",
  i AS inet, broadcast(i) AS "broadcast(inet)" FROM INET_TBL;
