INSERT INTO num_result SELECT id, 0, POW(numeric '10', LN(ABS(round(val,1000))))
    FROM num_data
    WHERE val != '0.0';
