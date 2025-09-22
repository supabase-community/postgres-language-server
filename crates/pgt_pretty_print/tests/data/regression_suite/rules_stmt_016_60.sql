create rule rtest_pers_upd as on update to rtest_person do also
	update rtest_admin set pname = new.pname where pname = old.pname;
