select id, f_timestamp, first_value(id) over w, last_value(id) over w
from datetimes
window w as (order by f_timestamp range between
             'infinity'::interval preceding and 'infinity'::interval following);
