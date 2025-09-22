select * from tuplesest_tab join
  (select b from tuplesest_parted where c < 100 group by b) sub
  on tuplesest_tab.a = sub.b;
