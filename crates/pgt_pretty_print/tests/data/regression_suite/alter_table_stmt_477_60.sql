alter table atacc1 add constraint atacc1_constr_or check(test_a is not null or test_b < 10);
