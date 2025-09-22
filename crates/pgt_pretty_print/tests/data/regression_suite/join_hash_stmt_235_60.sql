select length(max(s.t))
  from wide left join (select id, coalesce(t, '') || '' as t from wide) s using (id);
