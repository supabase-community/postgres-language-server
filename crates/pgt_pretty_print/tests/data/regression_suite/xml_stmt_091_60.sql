SELECT xmlserialize(DOCUMENT '<foo><bar><val x="y">42</val><val x="y">text node<val>73</val></val></bar></foo>' AS text INDENT);
