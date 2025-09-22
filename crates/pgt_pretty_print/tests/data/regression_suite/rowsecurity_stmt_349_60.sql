CREATE POLICY p2 ON document FOR INSERT WITH CHECK (dauthor = current_user);
