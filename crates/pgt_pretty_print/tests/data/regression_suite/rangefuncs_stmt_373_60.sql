SELECT * FROM ROWS FROM(generate_series(10,11), get_users()) WITH ORDINALITY;
