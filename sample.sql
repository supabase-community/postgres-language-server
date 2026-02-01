INSERT INTO indtoasttest (descr, f1, f2) VALUES ('two-compressed', repeat('1234567890', 1000), repeat('1234567890', 1000));

INSERT INTO public.country (code, continent_code, name, iso3, number, full_name) VALUES
  ('AF', 'AS', 'Afghanistan', 'AFG', '004', 'Islamic Republic of Afghanistan'),
  ('AX', 'EU', 'Åland Islands', 'ALA', '248', 'Åland Islands'),
  ('AL', 'EU', 'Albania', 'ALB', '008', 'Republic of Albania');

INSERT INTO indtoasttest (descr, f1, f2) VALUES (
    'two-compressed',
    repeat('1234567890', 1000),
    repeat('1234567890', 1000)
);

