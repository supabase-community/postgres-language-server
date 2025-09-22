create rule rtest_sys_del as on delete to rtest_system do also (
	delete from rtest_interface where sysname = old.sysname;
	delete from rtest_admin where sysname = old.sysname;
	);
