select description, inbytes, (test_conv(inbytes, 'mule_internal', 'euc_jp')).* from mic_inputs;
