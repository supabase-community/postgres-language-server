create table self_ref (a int primary key,
                       b int references self_ref(a) on delete cascade);
