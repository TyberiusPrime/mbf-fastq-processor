# Read names


FASTQ read names often consist of a unique ID and additional 'commentary' data.

These are often separated by a space - fastqrabs understanding can be configured via
[input.options.read_comment_character]({{< relref "docs/redirects/input-section.md" >}}#input-options).

To store data in this space, use [StoreTagInComment]({{< relref "docs/redirects/StoreTagInComment.md" >}}).

To manipulate existing comments, use [Rename]({{< relref "docs/redirects/Rename.md" >}}).


