#!/usr/sbin/ruby

img = IO.binread("./test.png")
esc = "\e{4;36;12{9;image/png{"
cmd = esc + img.length.to_s(16) + ";" + img + "}"
if ENV['TERM'] == 'natty'
    puts cmd
else
    puts "#{cmd.length} = #{img.length.to_s} + #{esc.length} + #{img.length.to_s(16).length} + 2"
end
