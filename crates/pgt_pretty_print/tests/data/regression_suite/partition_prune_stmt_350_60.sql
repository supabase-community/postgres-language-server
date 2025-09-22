prepare ab_q4 (int, int) as
select avg(a) from ab where a between $1 and $2 and b < 4;
