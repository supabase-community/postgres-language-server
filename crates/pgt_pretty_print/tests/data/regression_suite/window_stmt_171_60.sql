select id, f_time, first_value(id) over w, last_value(id) over w
from datetimes
window w as (order by f_time range between
             'infinity'::interval preceding and 'infinity'::interval preceding);
