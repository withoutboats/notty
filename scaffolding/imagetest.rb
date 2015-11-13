img = IO.binread('test.png')
puts "\x1b{4;12;8{9;image/png{" + img.length.to_s(16) + ';' + img + '}'
