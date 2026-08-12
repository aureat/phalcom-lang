import "./a" as A
import "./b" as B

A.User.new()./*@a*/aOnly()
B.User.new()./*@b*/bOnly()
