require 'base64'

img  = Base64.strict_encode64(IO.binread('test.png'))
mime = Base64.strict_encode64('image/png')
puts "\x1b_[14;12;8##{mime}##{img}\u{9c}"
