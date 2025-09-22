select tableoid::regclass as part, a, a%4 as "remainder = a % 4"
from hash_parted order by part;
