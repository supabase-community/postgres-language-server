select length(stringu1) into parallel_write
    from tenk1 group by length(stringu1);
