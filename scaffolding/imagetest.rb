require 'base64'

img  = Base64.strict_encode64(IO.binread('guernica.jpg'))
mime = Base64.strict_encode64('image/jpeg')
puts "\x1b_[14;32;8;4##{mime}##{img}\u{9c}"
