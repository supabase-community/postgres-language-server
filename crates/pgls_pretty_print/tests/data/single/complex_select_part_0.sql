  -- pk + fk referencing col
  select
    contype::text as contype,
    conname,
    array_length(conkey, 1) as ncol,
    conrelid as resorigtbl,
    col as resorigcol,
    ord
  from pg_constraint
  left join lateral unnest(conkey) with ordinality as _(col, ord) on true
  where contype IN ('p', 'f')
  union
  -- fk referenced col
  select
    concat(contype, '_ref') as contype,
    conname,
    array_length(confkey, 1) as ncol,
    confrelid,
    col,
    ord
  from pg_constraint
  left join lateral unnest(confkey) with ordinality as _(col, ord) on true
  where contype='f'

