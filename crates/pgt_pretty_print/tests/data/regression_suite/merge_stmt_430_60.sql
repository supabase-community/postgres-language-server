ALTER TABLE measurement ADD CONSTRAINT mcheck CHECK (city_id = 0) NO INHERIT;
