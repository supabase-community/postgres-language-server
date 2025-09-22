create table fktable2 (
  a int,
  b int,
  very_very_long_column_name_to_exceed_63_characters int,
  foreign key (very_very_long_column_name_to_exceed_63_characters) references pktable1,
  foreign key (a, very_very_long_column_name_to_exceed_63_characters) references pktable2,
  foreign key (a, very_very_long_column_name_to_exceed_63_characters) references pktable2
);
