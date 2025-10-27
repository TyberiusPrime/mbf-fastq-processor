status: open
# Profile code gen options


Hist.rs https://github.com/noamteyssier/hist-rs
for example uses this
[profile.release]
lto = true
codegen-units = 1
debug = false

We need to examine what these do and make a decision

