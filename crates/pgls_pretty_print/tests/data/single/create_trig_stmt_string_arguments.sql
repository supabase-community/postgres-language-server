-- A trigger argument that a word cannot spell is a string, not a quoted identifier, which the
-- parser would cut down to 63 bytes.
CREATE TRIGGER t BEFORE UPDATE ON t FOR EACH ROW EXECUTE PROCEDURE f('a, b.*', x, 1);
