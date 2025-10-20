status: open
# cLI integrated documentation


I want to show fragments of the template documentation on errors.


e.g. when we detect an error in ExtractRegions,
we show the documentation for that specific part of the template.

We'll first start by implementing 
an find_template(step) function, and hooking it behind 
the main template command, i.e. template ExtractRegions shows 
just that section.


