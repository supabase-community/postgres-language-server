CREATE FUNCTION on_login_proc() RETURNS event_trigger AS $$
BEGIN
  INSERT INTO user_logins (who) VALUES (SESSION_USER);
  RAISE NOTICE 'You are welcome!';
END;
$$ LANGUAGE plpgsql;
