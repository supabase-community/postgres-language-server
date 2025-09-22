select explain_parallel_append(
'select * from listp where a = (select 1)
  union all
select * from listp where a = (select 2);');
