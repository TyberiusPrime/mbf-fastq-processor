import gzip

fn_mine = "output_read1.fq.gz"

fn_merged_other = "ignore_fastp_output/merged.fastp.gz"


def get_reads(fn):
    lines = gzip.GzipFile(fn).read().decode().split("\n")
    reads = [
        {
            "name": lines[i],
            "seq": lines[i + 1],
            "qual": lines[i + 3],
        }
        for i in range(0, len(lines), 4) if lines[i]
    ]
    return reads


reads_mine= get_reads(fn_mine)
reads_merged_other = get_reads(fn_merged_other)

merged_other_by_name = {k['name'].split(" ",1)[0]: k for k in reads_merged_other}


count_merged_identical = 0
count_merged_different = 0
count_merged_different_qual = 0
count_unmerged_identical = 0
count_merged_but_not_in_other = 0
count_unmerged_but_in_other = 0

for read in reads_mine:
    is_merged = "XXX" not in read['seq']
    key = read['name'].split(" ",1)[0]
    other = merged_other_by_name.get(key, None)
    if is_merged and other:
        if other['seq'] == read['seq'] and other['qual'] == read['qual']:
            count_merged_identical += 1
        elif other['seq'] == read['seq']:
            count_merged_different_qual += 1
        else:
            print('differ', read['name'])
            count_merged_different += 1
    elif is_merged and not other:
        print('merged but should not be', read['name'])
        count_merged_but_not_in_other += 1
    elif not is_merged and other:
        print('unmerged but should be', read['name'])
        count_unmerged_but_in_other += 1
    elif not is_merged and not other:
        count_unmerged_identical += 1

print("Merged identical:", count_merged_identical)
print("Unmerged identical:", count_unmerged_identical)
print("total", count_merged_identical + count_unmerged_identical, 'out of', len(reads_mine))
print("---")
print("Merged different:", count_merged_different)
print("Merged different qual:", count_merged_different_qual)
print("Merged but not in other:", count_merged_but_not_in_other)
print("Unmerged but in other:", count_unmerged_but_in_other)


# Merged identical: 175176
# Unmerged identical: 4991
# ---
# Merged different: 26068
# Merged different qual: 0
# Merged but not in other: 4
# Unmerged but in other: 343



