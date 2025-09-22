alter table idxpart add exclude USING GIST (b with =, c with &&);
