status: open
# integrated documentation


I'm unhappy with having documentation in three places,
the source code (limited), the template.toml and the web / markdown docs.

I want to integrate at least the template.toml stuff into the source code,
so the CLI can say 'you had a error in step "ExtractRegion" - here's the doc,
in addition to the full template.


Implementation steps
 - replace the template.toml test case that reads the file with one that runs 
   `mbf-fastq-processor template`
 - introduce a fn documentation() -> Option<HashMap<String, String>> in the Step trait,
   return None by default.
 - have the template output function assemble all the documentations by iterating across
   all the Transformation enum variants (not listing them, macro magic).
   Sort them into sections by key 'Section'. Output their key 'brief'.
   If they return None, sort them into section 'Undocumented', and output
   a 'yet-to-document' string, which the test case will treat as an error.

   Within one section, order the variants by their 'weight' key (alphabetically, 
   use four digit weights. Followed by their name on ties.).
   
   The input/output sections get their own documentation() functions, 
   the order of sections is manually defined in the template output function.
   (With unknown sections raising an error.)

 - take the existing template.toml, slice it into the various steps,
   and insert them into the documentation() functions of each step.
   (Do this using a python script).

 -  When an error occurs, see if it contains '[input]', '[output]', or any '(<Step>)' 
    from the transformation enum, and if so, add it before the actual error output to the 
    messages. Have it sorted by weight by first occurrence, don't repeat sections.

