select explain_analyze('
update ab_a1 set b = 3 from ab where ab.a = 1 and ab.a = ab_a1.a;');
