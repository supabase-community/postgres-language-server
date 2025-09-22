select
  sum(unique1 order by ten, two), sum(unique1 order by four),
  sum(unique1 order by two, four)
from tenk1
group by ten;
