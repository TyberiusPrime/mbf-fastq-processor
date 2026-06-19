status: open
# Use cap instead of passing around directories


https://docs.rs/cap-std/latest/cap_std/index.html

Would prevent any TOCTOU on our file paths...

landlock might be an alternative though.
