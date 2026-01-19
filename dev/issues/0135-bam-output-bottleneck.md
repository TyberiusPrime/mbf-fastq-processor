status: open
# BAM output bottleneck


It runs at about 1/10 of the fastq output.

Is there a multi threaded writer maybe?

Or do we need to swap noodles for something better?
buffer sizes maybe?


-- 
Yeah, it's still terrible.
200k molecules vs 3.5 millions /s.


We might be able to 
either swap to libdeflate
(how to enable so it works?)
or multithread:

let mut writer = File::create(dst)
    .map(|f| bgzf::MultithreadedWriter::with_worker_count(worker_count, f))
    .map(bam::io::Writer::from)?;
