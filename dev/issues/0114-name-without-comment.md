status: closed
# name_without_comment

We should centralize the 'read-name-from-comment-separator'.

I think this warrants a final cleanup pass, both in concepts and the code.


- 
There is a subtlety here, since STAR for example will cut at the first space.
So using '|' and placing it *before* the read_comment_insert_char 
is sensible.

I'll leave it as is for now, but have fixed the default to be the one set in input.options

