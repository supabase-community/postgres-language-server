select * from ab where a = (select max(a) from lprt_a) and b = (select max(a)-1 from lprt_a);
