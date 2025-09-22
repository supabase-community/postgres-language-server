SELECT * FROM generate_series('2021-12-31 23:00:00+00'::timestamptz,
                              '2020-12-31 23:00:00+00'::timestamptz,
                              '-1 month'::interval,
                              'Europe/Warsaw');
