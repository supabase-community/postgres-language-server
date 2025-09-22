select * from (select pk,c2 from sq_limit order by c1,pk) as x limit 3;
