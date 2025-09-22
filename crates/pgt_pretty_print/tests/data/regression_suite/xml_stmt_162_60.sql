DO $$
DECLARE
  xml_declaration text := '<?xml version="1.0" encoding="ISO-8859-1"?>';
  degree_symbol text;
  res xml[];
BEGIN
  -- Per the documentation, except when the server encoding is UTF8, xpath()
  -- may not work on non-ASCII data.  The untranslatable_character and
  -- undefined_function traps below, currently dead code, will become relevant
  -- if we remove this limitation.
  IF current_setting('server_encoding') <> 'UTF8' THEN
    RAISE LOG 'skip: encoding % unsupported for xpath',
      current_setting('server_encoding');
    RETURN;
  END IF;

  degree_symbol := convert_from('\xc2b0', 'UTF8');
  res := xpath('text()', (xml_declaration ||
    '<x>' || degree_symbol || '</x>')::xml);
  IF degree_symbol <> res[1]::text THEN
    RAISE 'expected % (%), got % (%)',
      degree_symbol, convert_to(degree_symbol, 'UTF8'),
      res[1], convert_to(res[1]::text, 'UTF8');
  END IF;
EXCEPTION
  -- character with byte sequence 0xc2 0xb0 in encoding "UTF8" has no equivalent in encoding "LATIN8"
  WHEN untranslatable_character
  -- default conversion function for encoding "UTF8" to "MULE_INTERNAL" does not exist
  OR undefined_function
  -- unsupported XML feature
  OR feature_not_supported THEN
    RAISE LOG 'skip: %', SQLERRM;
END
$$;
