create table idxpart (a int4range, b int4range, c int4range, exclude USING GIST (a with =, b with =, c with &&)) partition by range (a, b);
