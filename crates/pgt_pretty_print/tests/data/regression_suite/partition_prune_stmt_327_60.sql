prepare hp_q1 (text) as
select * from hp where a is null and b = $1;
