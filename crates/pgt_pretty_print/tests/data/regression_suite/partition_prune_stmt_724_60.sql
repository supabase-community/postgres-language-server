with t as (
  merge into part_abc_view pt
  using (select stable_one() + 1 as pid) as q join part_abc_2 pt2 on (q.pid = pt2.a) on pt.a = stable_one() + 2
  when not matched then insert values (1, 'd', false) returning merge_action(), pt.*
)
insert into part_abc_log select * from t returning *;
