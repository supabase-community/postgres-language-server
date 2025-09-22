create domain d_fail as int constraint cc REFERENCES this_table_not_exists(i);
