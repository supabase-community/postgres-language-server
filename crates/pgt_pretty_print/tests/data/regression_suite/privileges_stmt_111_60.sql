SELECT session_user = current_role as c_r_ok, session_user = current_user as c_u_ok, current_setting('role') as role;
