create table p1_c1 (f1 int constraint f1_pos CHECK (f1 > 0)) inherits (p1);
