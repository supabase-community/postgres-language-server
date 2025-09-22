select id, f_interval, first_value(id) over w, last_value(id) over w
from datetimes
window w as (order by f_interval range between
             'infinity'::interval preceding and 'infinity'::interval preceding);
