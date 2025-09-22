create table p1p2_c2 (f1 int constraint f1_pos CHECK (f1 > 0)) inherits (p1, p2);
