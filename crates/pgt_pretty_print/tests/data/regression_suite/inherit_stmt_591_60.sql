create table inh_nn1 (f1 int check(f1 > 5) primary key references inh_nn1, f2 int not null);
