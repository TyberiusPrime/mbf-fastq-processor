# outcome: success
Getting started is complicated right now.
I want to implement a template functionality, 
that is a command line command that outputs a template
to standard output.

The template will have a default input block, 
every possible transformation (commented out, with explanations,
taken from the docs) and a default output block.


Claude, you need to 
    - put the existing command line behind a command (choose
      a sensible name).
    - introduce a new command line command called "template"
      that outputs the template to standard output.
    - the template get's stored in a text file in the source folder.
    - write an integration test for the existing command line command
      (use any of the test_cases as template for the test).
    - write a python script that assembles the template from the docs
      (docs/content/docs/reference folder). 
      Order: input section, demultiplex, tag steps, filter steps, modification steps,
      report steps, options, output section.
      Separate the sections with a clear, regexable heading.
    - write a test case that splits the template into 'per step' sections (on the heading),
      then creates a simple config with input and the step, 
      and verifies tha it parses correctly.

Please perform a jj commit after every step and bookmark the final commit
with 'template'
     




