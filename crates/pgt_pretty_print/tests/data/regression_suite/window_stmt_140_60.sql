select x, last_value(x) over (order by x range between current row and 4 following)
from generate_series(2147483644, 2147483646) x;
