-- A string constant longer than an identifier stays a string: written as a word, the parser
-- would cut it down to 63 bytes.
SET my.option TO 'nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn';
