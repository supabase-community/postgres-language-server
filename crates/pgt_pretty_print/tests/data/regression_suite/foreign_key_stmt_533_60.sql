create table fktable2 (x float8, y float8, foreign key (x, y) references pktable2 (a, b) on update cascade);
