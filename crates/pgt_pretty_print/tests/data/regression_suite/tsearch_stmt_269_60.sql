SELECT ts_headline('english',
'Lorem ipsum urna.  Nullam nullam ullamcorper urna.',
phraseto_tsquery('english','ullamcorper urna'),
'MaxWords=100, MinWords=5');
