CREATE FUNCTION mood_to_json(mood) RETURNS json AS $$
  SELECT to_json($1::text);
$$ LANGUAGE sql IMMUTABLE;
