select percentile_cont(array[0,0.25,0.5,0.75,1]) within group (order by thousand)
from tenk1;
