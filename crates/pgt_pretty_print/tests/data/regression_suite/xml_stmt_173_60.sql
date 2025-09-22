SELECT COUNT(id) FROM xmltest WHERE xmlexists('/menu/beer' PASSING data);
