select
  max(four order by four), sum(two order by two),
  min(four order by four), max(two order by two),
  sum(ten order by ten), min(ten order by ten), max(ten order by ten)
from tenk1;
