select id, f_timestamptz, first_value(id) over w, last_value(id) over w
from datetimes
window w as (order by f_timestamptz range between
             'infinity'::interval preceding and 'infinity'::interval following);
