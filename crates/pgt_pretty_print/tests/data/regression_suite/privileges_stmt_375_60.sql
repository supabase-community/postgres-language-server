INSERT INTO atest5(three) VALUES (4) ON CONFLICT (two) DO UPDATE set three = 10;
