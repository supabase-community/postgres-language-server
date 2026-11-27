create table   if not exists tenant_%s (id bigint primary key,  note text);
select * from app_%s.events_%d, "%s"   where x = %v;
