select
  max(four order by four), sum(two order by two),
  min(four order by four), max(two order by two)
from tenk1;
