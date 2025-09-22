INSERT INTO vac_option_tab SELECT a, 't' || a FROM generate_series(1, 10) AS a;
