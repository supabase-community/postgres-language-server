INSERT INTO temporal_mltrng (id, valid_at) VALUES
    ('[5,6)', datemultirange(daterange('2018-01-01', '2018-02-01'))),
    ('[5,6)', datemultirange(daterange('2018-02-01', '2018-03-01')));
