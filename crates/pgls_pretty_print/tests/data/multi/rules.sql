create table rtest_t1 (a int4, b int4);

create table rtest_t2 (a int4, b int4);

create table rtest_t3 (a int4, b int4);

create view rtest_v1 as select * from rtest_t1;

create rule rtest_v1_ins as on insert to rtest_v1 do instead
	insert into rtest_t1 values (new.a, new.b);

create rule rtest_v1_upd as on update to rtest_v1 do instead
	update rtest_t1 set a = new.a, b = new.b
	where a = old.a;

create rule rtest_v1_del as on delete to rtest_v1 do instead
	delete from rtest_t1 where a = old.a;

COMMENT ON RULE rtest_v1_bad ON rtest_v1 IS 'bad rule';

COMMENT ON RULE rtest_v1_del ON rtest_v1 IS 'delete rule';

COMMENT ON RULE rtest_v1_del ON rtest_v1 IS NULL;

create table rtest_system (sysname text, sysdesc text);

create table rtest_interface (sysname text, ifname text);

create table rtest_person (pname text, pdesc text);

create table rtest_admin (pname text, sysname text);

create rule rtest_sys_upd as on update to rtest_system do also (
	update rtest_interface set sysname = new.sysname
		where sysname = old.sysname;
	update rtest_admin set sysname = new.sysname
		where sysname = old.sysname
	);

create rule rtest_sys_del as on delete to rtest_system do also (
	delete from rtest_interface where sysname = old.sysname;
	delete from rtest_admin where sysname = old.sysname;
	);

create rule rtest_pers_upd as on update to rtest_person do also
	update rtest_admin set pname = new.pname where pname = old.pname;

create rule rtest_pers_del as on delete to rtest_person do also
	delete from rtest_admin where pname = old.pname;

create table rtest_emp (ename char(20), salary numeric);

create table rtest_emplog (ename char(20), who name, action char(10), newsal numeric, oldsal numeric);

create table rtest_empmass (ename char(20), salary numeric);

create rule rtest_emp_ins as on insert to rtest_emp do
	insert into rtest_emplog values (new.ename, current_user,
			'hired', new.salary, '0.00');

create rule rtest_emp_upd as on update to rtest_emp where new.salary != old.salary do
	insert into rtest_emplog values (new.ename, current_user,
			'honored', new.salary, old.salary);

create rule rtest_emp_del as on delete to rtest_emp do
	insert into rtest_emplog values (old.ename, current_user,
			'fired', '0.00', old.salary);

create table rtest_t4 (a int4, b text);

create table rtest_t5 (a int4, b text);

create table rtest_t6 (a int4, b text);

create table rtest_t7 (a int4, b text);

create table rtest_t8 (a int4, b text);

create table rtest_t9 (a int4, b text);

create rule rtest_t4_ins1 as on insert to rtest_t4
		where new.a >= 10 and new.a < 20 do instead
	insert into rtest_t5 values (new.a, new.b);

create rule rtest_t4_ins2 as on insert to rtest_t4
		where new.a >= 20 and new.a < 30 do
	insert into rtest_t6 values (new.a, new.b);

create rule rtest_t5_ins as on insert to rtest_t5
		where new.a > 15 do
	insert into rtest_t7 values (new.a, new.b);

create rule rtest_t6_ins as on insert to rtest_t6
		where new.a > 25 do instead
	insert into rtest_t8 values (new.a, new.b);

create table rtest_order1 (a int4);

create table rtest_order2 (a int4, b int4, c text);

create sequence rtest_seq;

create rule rtest_order_r3 as on insert to rtest_order1 do instead
	insert into rtest_order2 values (new.a, nextval('rtest_seq'),
		'rule 3 - this should run 3rd');

create rule rtest_order_r4 as on insert to rtest_order1
		where a < 100 do instead
	insert into rtest_order2 values (new.a, nextval('rtest_seq'),
		'rule 4 - this should run 4th');

create rule rtest_order_r2 as on insert to rtest_order1 do
	insert into rtest_order2 values (new.a, nextval('rtest_seq'),
		'rule 2 - this should run 2nd');

create rule rtest_order_r1 as on insert to rtest_order1 do instead
	insert into rtest_order2 values (new.a, nextval('rtest_seq'),
		'rule 1 - this should run 1st');

create table rtest_nothn1 (a int4, b text);

create table rtest_nothn2 (a int4, b text);

create table rtest_nothn3 (a int4, b text);

create table rtest_nothn4 (a int4, b text);

create rule rtest_nothn_r1 as on insert to rtest_nothn1
	where new.a >= 10 and new.a < 20 do instead nothing;

create rule rtest_nothn_r2 as on insert to rtest_nothn1
	where new.a >= 30 and new.a < 40 do instead nothing;

create rule rtest_nothn_r3 as on insert to rtest_nothn2
	where new.a >= 100 do instead
	insert into rtest_nothn3 values (new.a, new.b);

create rule rtest_nothn_r4 as on insert to rtest_nothn2
	do instead nothing;

insert into rtest_t2 values (1, 21);

insert into rtest_t2 values (2, 22);

insert into rtest_t2 values (3, 23);

insert into rtest_t3 values (1, 31);

insert into rtest_t3 values (2, 32);

insert into rtest_t3 values (3, 33);

insert into rtest_t3 values (4, 34);

insert into rtest_t3 values (5, 35);

insert into rtest_v1 values (1, 11);

insert into rtest_v1 values (2, 12);

select * from rtest_v1;

delete from rtest_v1 where a = 1;

select * from rtest_v1;

insert into rtest_v1 values (1, 11);

delete from rtest_v1 where b = 12;

select * from rtest_v1;

insert into rtest_v1 values (2, 12);

insert into rtest_v1 values (2, 13);

select * from rtest_v1;
