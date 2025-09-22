select min(a) over (partition by a order by a) from part_abc where a >= stable_one() + 1 and d <= stable_one()
union all
select min(a) over (partition by a order by a) from part_abc where a >= stable_one() + 1 and d >= stable_one();
