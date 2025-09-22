select id, f_timestamp, first_value(id) over w, last_value(id) over w
from datetimes
window w as (order by f_timestamp range between
             '-infinity'::interval following and
             'infinity'::interval following);
