CREATE TABLE measurement (
    id integer,
    logdate date
) PARTITION BY RANGE (logdate)