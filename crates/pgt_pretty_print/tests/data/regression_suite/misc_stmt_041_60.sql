SELECT p.name, name(p.hobbies), name(equipment(p.hobbies)) FROM ONLY person p;
