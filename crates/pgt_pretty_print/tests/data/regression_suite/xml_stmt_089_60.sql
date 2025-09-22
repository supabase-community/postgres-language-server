SELECT xmlserialize(DOCUMENT 'text node<foo>73</foo>text node<bar><val x="y">42</val></bar>' AS text INDENT);
