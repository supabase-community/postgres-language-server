create table fktable(ftest1 cidr, ftest2 int[], foreign key (ftest1, ftest2) references pktable);
