SELECT f1,
    date_part('microsecond', f1) AS microsecond,
    date_part('millisecond', f1) AS millisecond,
    date_part('second', f1) AS second,
    date_part('epoch', f1) AS epoch
    FROM INTERVAL_TBL;
