-- A provider longer than an identifier stays a string: written as a word, the parser would cut it
-- down to 63 bytes.
SECURITY LABEL FOR 'a provider name with spaces that is longer than the sixty-three bytes an identifier keeps, and longer than a line' ON TABLE t IS 'classified';
