select explain_analyze('
update ab_a1 set b = 3 from ab_a2 where ab_a2.b = (select 1);');
