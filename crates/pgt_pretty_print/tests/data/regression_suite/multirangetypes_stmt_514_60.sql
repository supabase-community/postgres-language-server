create domain restrictedmultirange as int4multirange check (upper(value) < 10);
