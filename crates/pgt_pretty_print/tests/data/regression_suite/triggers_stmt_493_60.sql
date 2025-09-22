create table parted_irreg (fd int, a int, fd2 int, b text)
  partition by range (b);
