CREATE TEXT SEARCH DICTIONARY ispell (
                        Template=ispell,
                        DictFile=ispell_sample,
                        AffFile=ispell_sample
);

SELECT ts_lexize('ispell', 'skies');

SELECT ts_lexize('ispell', 'bookings');

SELECT ts_lexize('ispell', 'booking');

SELECT ts_lexize('ispell', 'foot');

SELECT ts_lexize('ispell', 'foots');

SELECT ts_lexize('ispell', 'rebookings');

SELECT ts_lexize('ispell', 'rebooking');

SELECT ts_lexize('ispell', 'rebook');

SELECT ts_lexize('ispell', 'unbookings');

SELECT ts_lexize('ispell', 'unbooking');

SELECT ts_lexize('ispell', 'unbook');

SELECT ts_lexize('ispell', 'footklubber');

SELECT ts_lexize('ispell', 'footballklubber');

SELECT ts_lexize('ispell', 'ballyklubber');

SELECT ts_lexize('ispell', 'footballyklubber');

CREATE TEXT SEARCH DICTIONARY hunspell (
                        Template=ispell,
                        DictFile=ispell_sample,
                        AffFile=hunspell_sample
);

SELECT ts_lexize('hunspell', 'skies');

SELECT ts_lexize('hunspell', 'bookings');

SELECT ts_lexize('hunspell', 'booking');

SELECT ts_lexize('hunspell', 'foot');

SELECT ts_lexize('hunspell', 'foots');

SELECT ts_lexize('hunspell', 'rebookings');

SELECT ts_lexize('hunspell', 'rebooking');

SELECT ts_lexize('hunspell', 'rebook');

SELECT ts_lexize('hunspell', 'unbookings');

SELECT ts_lexize('hunspell', 'unbooking');

SELECT ts_lexize('hunspell', 'unbook');

SELECT ts_lexize('hunspell', 'footklubber');

SELECT ts_lexize('hunspell', 'footballklubber');

SELECT ts_lexize('hunspell', 'ballyklubber');

SELECT ts_lexize('hunspell', 'footballyklubber');

CREATE TEXT SEARCH DICTIONARY hunspell_long (
                        Template=ispell,
                        DictFile=hunspell_sample_long,
                        AffFile=hunspell_sample_long
);

SELECT ts_lexize('hunspell_long', 'skies');

SELECT ts_lexize('hunspell_long', 'bookings');

SELECT ts_lexize('hunspell_long', 'booking');

SELECT ts_lexize('hunspell_long', 'foot');

SELECT ts_lexize('hunspell_long', 'foots');

SELECT ts_lexize('hunspell_long', 'rebookings');

SELECT ts_lexize('hunspell_long', 'rebooking');

SELECT ts_lexize('hunspell_long', 'rebook');

SELECT ts_lexize('hunspell_long', 'unbookings');

SELECT ts_lexize('hunspell_long', 'unbooking');

SELECT ts_lexize('hunspell_long', 'unbook');

SELECT ts_lexize('hunspell_long', 'booked');

SELECT ts_lexize('hunspell_long', 'footklubber');

SELECT ts_lexize('hunspell_long', 'footballklubber');

SELECT ts_lexize('hunspell_long', 'ballyklubber');

SELECT ts_lexize('hunspell_long', 'ballsklubber');

SELECT ts_lexize('hunspell_long', 'footballyklubber');

SELECT ts_lexize('hunspell_long', 'ex-machina');

CREATE TEXT SEARCH DICTIONARY hunspell_num (
                        Template=ispell,
                        DictFile=hunspell_sample_num,
                        AffFile=hunspell_sample_num
);

SELECT ts_lexize('hunspell_num', 'skies');

SELECT ts_lexize('hunspell_num', 'sk');

SELECT ts_lexize('hunspell_num', 'bookings');

SELECT ts_lexize('hunspell_num', 'booking');

SELECT ts_lexize('hunspell_num', 'foot');

SELECT ts_lexize('hunspell_num', 'foots');

SELECT ts_lexize('hunspell_num', 'rebookings');

SELECT ts_lexize('hunspell_num', 'rebooking');

SELECT ts_lexize('hunspell_num', 'rebook');

SELECT ts_lexize('hunspell_num', 'unbookings');

SELECT ts_lexize('hunspell_num', 'unbooking');

SELECT ts_lexize('hunspell_num', 'unbook');

SELECT ts_lexize('hunspell_num', 'booked');

SELECT ts_lexize('hunspell_num', 'footklubber');

SELECT ts_lexize('hunspell_num', 'footballklubber');

SELECT ts_lexize('hunspell_num', 'ballyklubber');

SELECT ts_lexize('hunspell_num', 'footballyklubber');

CREATE TEXT SEARCH DICTIONARY hunspell_err (
						Template=ispell,
						DictFile=ispell_sample,
						AffFile=hunspell_sample_long
);

CREATE TEXT SEARCH DICTIONARY hunspell_err (
						Template=ispell,
						DictFile=ispell_sample,
						AffFile=hunspell_sample_num
);

CREATE TEXT SEARCH DICTIONARY hunspell_invalid_1 (
						Template=ispell,
						DictFile=hunspell_sample_long,
						AffFile=ispell_sample
);

CREATE TEXT SEARCH DICTIONARY hunspell_invalid_2 (
						Template=ispell,
						DictFile=hunspell_sample_long,
						AffFile=hunspell_sample_num
);

CREATE TEXT SEARCH DICTIONARY hunspell_invalid_3 (
						Template=ispell,
						DictFile=hunspell_sample_num,
						AffFile=ispell_sample
);

CREATE TEXT SEARCH DICTIONARY hunspell_err (
						Template=ispell,
						DictFile=hunspell_sample_num,
						AffFile=hunspell_sample_long
);

CREATE TEXT SEARCH DICTIONARY synonym (
						Template=synonym,
						Synonyms=synonym_sample
);

SELECT ts_lexize('synonym', 'PoStGrEs');

SELECT ts_lexize('synonym', 'Gogle');

SELECT ts_lexize('synonym', 'indices');

SELECT dictinitoption FROM pg_ts_dict WHERE dictname = 'synonym';

ALTER TEXT SEARCH DICTIONARY synonym (CaseSensitive = 1);

SELECT ts_lexize('synonym', 'PoStGrEs');

SELECT dictinitoption FROM pg_ts_dict WHERE dictname = 'synonym';

ALTER TEXT SEARCH DICTIONARY synonym (CaseSensitive = 2);

ALTER TEXT SEARCH DICTIONARY synonym (CaseSensitive = off);

SELECT ts_lexize('synonym', 'PoStGrEs');

SELECT dictinitoption FROM pg_ts_dict WHERE dictname = 'synonym';

CREATE TEXT SEARCH DICTIONARY thesaurus (
                        Template=thesaurus,
						DictFile=thesaurus_sample,
						Dictionary=english_stem
);

SELECT ts_lexize('thesaurus', 'one');

CREATE TEXT SEARCH CONFIGURATION ispell_tst (
						COPY=english
);
