alter table idxpart add exclude USING GIST (a with =, b with =, c with &&);
