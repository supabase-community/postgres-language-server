CREATE TABLE follower (a int REFERENCES leader ON DELETE CASCADE, b int);
