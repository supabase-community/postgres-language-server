create table idxpart (a int4range, b int4range, exclude USING GIST (a with = )) partition by range (a, b);
