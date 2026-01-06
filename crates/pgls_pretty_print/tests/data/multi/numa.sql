SELECT NOT(pg_numa_available()) AS skip_test ;

SELECT COUNT(*) = 0 AS ok FROM pg_shmem_allocations_numa;

SELECT COUNT(*) >= 0 AS ok FROM pg_shmem_allocations_numa;
