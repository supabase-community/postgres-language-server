create table idxpart (a int4range, b int4range, c int4range, exclude USING GIST (b with =, c with &&)) partition by range (a);
