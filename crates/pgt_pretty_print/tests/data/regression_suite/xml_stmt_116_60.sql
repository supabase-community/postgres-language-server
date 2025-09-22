SELECT xmlelement(name employees, xmlagg(xmlelement(name name, name))) FROM emp;
