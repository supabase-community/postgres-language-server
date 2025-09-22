SELECT upper(c collate case_insensitive), count(c) FROM pagg_tab3 GROUP BY c collate case_insensitive ORDER BY 1;
