-- A trigger argument longer than an identifier stays a string.
CREATE TRIGGER t BEFORE UPDATE ON t FOR EACH ROW EXECUTE PROCEDURE f('nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn');
