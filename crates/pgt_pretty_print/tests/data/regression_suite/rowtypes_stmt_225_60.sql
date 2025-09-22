select (ss.a).x, (ss.a).n from
  (select information_schema._pg_expandarray(array[1,2]) AS a) ss
where false;
