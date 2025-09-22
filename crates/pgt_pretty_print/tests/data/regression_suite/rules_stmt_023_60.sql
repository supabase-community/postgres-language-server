create rule rtest_emp_del as on delete to rtest_emp do
	insert into rtest_emplog values (old.ename, current_user,
			'fired', '0.00', old.salary);
