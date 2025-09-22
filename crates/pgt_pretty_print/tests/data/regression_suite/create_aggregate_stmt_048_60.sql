create function sum4(int8,int8,int8,int8) returns int8 as
'select $1 + $2 + $3 + $4' language sql strict immutable;
