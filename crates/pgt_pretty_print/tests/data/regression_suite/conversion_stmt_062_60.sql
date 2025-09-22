select description, inbytes, (test_conv(inbytes, 'mule_internal', 'sjis')).* from mic_inputs;
