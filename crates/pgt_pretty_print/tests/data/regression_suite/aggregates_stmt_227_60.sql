select
  sum(two order by two), max(four order by four),
  min(four order by four), max(two order by two)
from tenk1;
