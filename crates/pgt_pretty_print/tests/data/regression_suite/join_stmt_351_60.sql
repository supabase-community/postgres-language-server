select * from
     (select k from
         (select i, coalesce(i, j) as k from
             (select i from t union all select 0)
             join (select 1 as j limit 1) on i = j)
         right join (select 2 as x) on true
         join (select 3 as y) on i is not null
     ),
     lateral (select k as kl limit 1);
