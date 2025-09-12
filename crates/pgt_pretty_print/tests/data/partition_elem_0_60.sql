CREATE TABLE measurement (
    city_id int,
    logdate date
) PARTITION BY RANGE (logdate)