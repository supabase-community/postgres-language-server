SELECT xmlelement(name employee, xmlforest(name, age, salary as pay)) FROM emp;
