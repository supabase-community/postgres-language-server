prepare ps1 as
  select * from mc3p where a = $1 and abs(b) < (select 3);
