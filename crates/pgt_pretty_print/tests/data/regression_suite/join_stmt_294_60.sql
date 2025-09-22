create temp table b (
     a char not null,
     num integer not null,
     constraint b_pk primary key (a, num)
);
