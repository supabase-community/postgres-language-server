select ten, grouping(ten) from onek
group by cube(ten) having grouping(ten) > 0
order by 2,1;
