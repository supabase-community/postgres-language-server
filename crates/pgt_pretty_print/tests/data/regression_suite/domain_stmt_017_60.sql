create domain d_fail as int4 constraint cc check (values > 1) deferrable;
