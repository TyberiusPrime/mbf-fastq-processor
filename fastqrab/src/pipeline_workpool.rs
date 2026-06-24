/// A workpool based pipeline
/// Main advantage is a tight control over the number of blocks in flight,
/// working premature termination (head...)
/// Automatic work distribution & multi coring of non-serial stages
/// No over subscription (like the old n threads per non-serial stage + 1 thread per serial stage model)
///
use anyhow::{Result, bail};
use crossbeam::channel::{Receiver, Sender, select};
use std::sync::{Arc, Mutex};

use crate::{
    config::Stage,
    demultiplex::OptDemultiplex,
    transformations::{self, Step},
};
use bstr::BString;
use fastqrab_config::{
    TagLabel,
    dna::{StringColumn, TagColumn},
};
use fastqrab_io::{blocks::FastQChunk, io};
use stringpod::{Lifted, RegionLift};

pub struct WorkItem {
    pub block: io::FastQBlocksCombined,
    pub expected_read_count: Option<usize>,
    pub stage_index: usize,
}

#[derive(Clone)]
pub struct BlockStatus {
    pub current_stage: usize,
    pub block: io::FastQBlocksCombined,
    pub expected_read_count: Option<usize>,
}

pub struct StageProgress {
    pub highest_completed_block: usize,
    pub needs_serial: bool,
    pub transmits_premature_termination: bool,
    pub closed: bool,
}

pub struct WorkResult {
    pub work_item: WorkItem,
    pub stage_continue: bool,
    pub error: Option<anyhow::Error>,
}

pub struct WorkpoolCoordinator {
    stages: Vec<Arc<Stage>>,
    stage_progress: Vec<StageProgress>,

    stalled_blocks: Option<Vec<BlockStatus>>, //blocks waiting to get ready.

    current_blocks_in_flight: usize, // that's 'within pipeline, stalled + currently being worked on.
    /// Sum of the *admitted* read counts of all blocks currently in flight.
    /// Drives backpressure (`max_molecules_in_flight`). We record the read count a
    /// block had at admission and release exactly that amount when it leaves,
    /// because stages (e.g. Head) truncate blocks mid-pipeline.
    current_reads_in_flight: usize,
    max_molecules_in_flight: usize,
    /// Shared mirror of `current_blocks_in_flight`, published once per loop tick
    /// so the `Progress` step can report pipeline occupancy. Diagnostic only.
    blocks_in_flight_gauge: Arc<std::sync::atomic::AtomicUsize>,

    incoming_rx: Option<Receiver<(io::FastQBlocksCombined, Option<usize>)>>,
    todo_tx: Sender<WorkItem>,     //towards workers
    done_rx: Receiver<WorkResult>, //back from workers
    output_tx: Sender<(io::FastQBlocksCombined, Option<usize>)>,
    output_done_rx: Receiver<crate::pipeline::BlockFinallyDone>,

    report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,

    last_incoming_block: Option<usize>,
}

enum CanTake {
    Yes,
    No,
    Drop,
}

impl WorkpoolCoordinator {
    #[expect(clippy::too_many_arguments, reason = "needed")]
    pub fn new(
        stages: Vec<Stage>,
        max_molecules_in_flight: usize,
        incoming_rx: Receiver<(io::FastQBlocksCombined, Option<usize>)>,
        todo_tx: Sender<WorkItem>,     //towards workers
        done_rx: Receiver<WorkResult>, //back from workers
        output_tx: Sender<(io::FastQBlocksCombined, Option<usize>)>,
        output_done_rx: Receiver<crate::pipeline::BlockFinallyDone>,

        report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
        error_collector: Arc<Mutex<Vec<String>>>,
        blocks_in_flight_gauge: Arc<std::sync::atomic::AtomicUsize>,
    ) -> (Self, Vec<Arc<Stage>>) {
        let stage_progress: Vec<StageProgress> = stages
            .iter()
            .map(|stage| StageProgress {
                highest_completed_block: 0,
                needs_serial: stage.transformation.needs_serial(),
                closed: false,
                transmits_premature_termination: stage
                    .transformation
                    .transmits_premature_termination(),
            })
            .collect();

        let arc_stages: Vec<Arc<Stage>> = stages.into_iter().map(Arc::new).collect();

        let stages_for_workers = arc_stages.clone();

        let coordinator = Self {
            stages: arc_stages,
            stage_progress,
            stalled_blocks: Some(Vec::new()),
            max_molecules_in_flight,
            current_blocks_in_flight: 0,
            current_reads_in_flight: 0,
            blocks_in_flight_gauge,

            incoming_rx: Some(incoming_rx),
            todo_tx,
            done_rx,
            output_tx,
            output_done_rx,

            error_collector,
            report_collector,
            last_incoming_block: None,
        };

        (coordinator, stages_for_workers)
    }

    #[expect(clippy::too_many_lines, reason = "needed")]
    pub fn run(mut self, demultiplex_infos: &[(usize, OptDemultiplex)]) {
        loop {
            // Publish current occupancy for the Progress step's diagnostic gauge.
            self.blocks_in_flight_gauge.store(
                self.current_blocks_in_flight,
                std::sync::atomic::Ordering::Relaxed,
            );
            // Check if we're at capacity. We admit whole blocks while still
            // under the read budget, so we may overshoot by up to one block.
            // (A read-based budget instead of a block count keeps memory bounded
            // regardless of block_size, and never stalls a block-granular
            // pre-fetch such as HammingCorrect's ByMajority warm-up.)
            let accept_new_incoming = self.current_reads_in_flight < self.max_molecules_in_flight;
            if self.incoming_rx.is_none() || !accept_new_incoming {
                // Only listen for completed work when input is closed
                select! {
                    recv(self.done_rx) -> msg => {
                        match msg {
                            //match done_rx.recv_timeout(std::time::Duration::from_millis(1000)) {
                            Ok(work_result) => {
                                if self.process_completed_work(work_result).is_err() {
                                    break; // Coordinator decided to terminate because of an error.
                                }
                            }
                            Err(_) => {
                                // cov:excl-start
                                self
                                .error_collector
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .push(
                                    "No incoming blocks and no completed work; terminating coordinator."
                                        .to_string(),
                                );
                                break; // WoSleep::rkers closed
                                // cov:excl-stop
                            }
                        }
                    }
                    recv(self.output_done_rx) -> msg => {
                        match msg {
                            Ok(msg) => {
                                self.release_block(msg.initial_molecule_count);
                            }
                            Err(_) => {
                                // Output pipe crashed?
                                self
                                    .error_collector
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .push(
                                        "Output pipe closed unexpectedly; terminating coordinator."
                                            .to_string(),
                                    );
                                break;
                            }
                        }
                    }
                }
            } else {
                // Listen for both incoming and done messages
                select! {
                    recv(self.incoming_rx.as_ref().expect("Checked for someness just before")) -> msg => {
                        match msg {
                            Ok((block, expected_read_count)) => {
                                if self.process_incoming_block(block.block_no(), block, expected_read_count).is_err() {
                                    // cov:excl-start
                                    break
                                    // cov:excl-stop
                                }
                            }
                            Err(_) => {

                                {  // drop it so it will fail earlier, not filling it's buffer
                                    self.incoming_rx.take();
                                }
                                // Continue processing to handle remaining work
                            }
                        }
                    }

                    recv(self.done_rx) -> msg => {
                        match msg {
                            Ok(work_result) => {
                                if self.process_completed_work(work_result).is_err() {
                                    // Coordinator decided to terminate because of an error.
                                    break;  //cov:excl-line timing dependent
                                }
                            }
                            Err(_) => {
                                // cov:excl-start
                                break; // Workers pipe crashed?
                                // cov:excl-stop
                            }
                        }
                    }
                    recv(self.output_done_rx) -> msg => {
                        match msg {
                            Ok(msg_finally_done) => {
                                self.release_block(msg_finally_done.initial_molecule_count);
                            }
                            Err(_) => {
                                // Output pipe crashed?
                                // cov:excl-start
                                self
                                    .error_collector
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .push(
                                        "Output pipe closed unexpectedly; terminating coordinator."
                                            .to_string(),
                                    );
                                break;
                                // cov:excl-stop
                            }
                        }
                    }
                }

                if !self.stages.is_empty() && self.stage_progress[0].closed {
                    {
                        // drop it so it will fail earlier, not filling it's buffer
                        self.incoming_rx.take();
                    }
                }
            }

            // Check if we should terminate
            // eprintln!(
            //     "Current in-flight: {}, pending work items: {}, stalled blocks: {}, input open: {}",
            //     self.current_blocks_in_flight,
            //     self.pending_work_items,
            //     self.stalled_blocks.as_ref().unwrap().len(),
            //     self.incoming_rx.is_some()
            // );
            if self.incoming_rx.is_none()
                && self.stalled_blocks.as_ref().expect("Should never be none outside of queue_stalled, and there only for borrow checker workaround").is_empty()
                && self.current_blocks_in_flight == 0
            {
                break;
            }
        }

        // Finalize reports before ending
        self.finalize_reports(demultiplex_infos);
    }

    pub fn process_incoming_block(
        &mut self,
        block_no: usize,
        block: io::FastQBlocksCombined,
        expected_read_count: Option<usize>,
    ) -> Result<()> {
        // eprintln!("Adding to pipeline: {block_no}");
        if let Some(last_block_no) = self.last_incoming_block {
            assert!(
                block_no == last_block_no + 1,
                "Bug: Incoming block numbers are not sequential"
            );
        }

        self.last_incoming_block = Some(block_no);
        let admitted_reads = block.len();
        let block_status = BlockStatus {
            current_stage: 0,
            block,
            expected_read_count,
        };
        self.current_blocks_in_flight += 1;
        self.current_reads_in_flight += admitted_reads;
        self.queue_block(block_status)?;
        Ok(())
    }

    /// A block has left the pipeline (output, dropped at a closed stage, or
    /// discarded after the stage that closed it). Release the bookkeeping we
    /// took at admission — exactly the read count it had then, since stages may
    /// have truncated it since.
    fn release_block(&mut self, initial_molecule_count: usize) {
        self.current_blocks_in_flight -= 1;
        self.current_reads_in_flight -= initial_molecule_count;
    }

    fn queue_block(&mut self, block_status: BlockStatus) -> Result<()> {
        if self.stages.is_empty() {
            self.output_block(block_status)?;
        } else {
            match Self::stage_can_take_block(
                &self.stage_progress,
                block_status.current_stage,
                block_status.block.block_no(),
            ) {
                CanTake::Yes => {
                    // eprintln!("Sending block {} off to process stage {}", block_status.block_no, block_status.current_stage);
                    self.send_block_to_workers(block_status)?;
                }
                CanTake::No => {
                    self.stalled_blocks
                        .as_mut()
                        .expect("Should never be none outside queue_stalled")
                        .push(block_status);
                }
                CanTake::Drop => {
                    // eprintln!(
                    //     "Dropping after stage: block {} (next stage was {}",
                    //     block_status.block_no, block_status.current_stage
                    // );
                    self.release_block(block_status.block.initial_molecule_count()); // we drop it here
                }
            }
        }
        Ok(())
    }

    fn stage_can_take_block(
        stage_progress: &[StageProgress],
        stage_index: usize,
        block_no: usize,
    ) -> CanTake {
        if stage_progress[stage_index].closed {
            CanTake::Drop
        } else if !stage_progress[stage_index].needs_serial {
            //fp in mutation testing.
            CanTake::Yes
        } else if stage_progress[stage_index].highest_completed_block + 1 == block_no {
            CanTake::Yes
        } else {
            CanTake::No
        }
    }

    pub fn send_block_to_workers(&mut self, block_status: BlockStatus) -> Result<()> {
        let block_no = block_status.block.block_no();
        let work_item = WorkItem {
            block: block_status.block,
            expected_read_count: block_status.expected_read_count,
            stage_index: block_status.current_stage,
        };
        if self.todo_tx.send(work_item).is_ok() {
            Ok(())
        } else {
            // cov:excl-start
            bail!("Failed to send work item for block {block_no}");
            // cov:excl-stop
        }
    }

    pub fn process_completed_work(&mut self, work_result: WorkResult) -> Result<()> {
        let block_no = work_result.work_item.block.block_no();
        let stage_index = work_result.work_item.stage_index;

        // eprintln!(
        //     "Completed stage {} for block {}. Continue: {}",
        //     stage_index, block_no, work_result.stage_continue
        // );

        // Update stage progress
        if self.stage_progress[stage_index].highest_completed_block < block_no {
            self.stage_progress[stage_index].highest_completed_block = block_no;
        }

        if let Some(error) = work_result.error {
            // Handle error - for now, continue pipeline with empty block
            self.error_collector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(format!("Error in stage {stage_index}: {error:?}"));
            bail!("error detected");
        }

        // Create or update block status
        let mut block_status = BlockStatus {
            current_stage: stage_index + 1,
            block: work_result.work_item.block,
            expected_read_count: work_result.work_item.expected_read_count,
        };

        let was_already_closed = self.stage_progress[stage_index].closed;
        if !work_result.stage_continue {
            // Stage requested premature termination - mark block as final
            block_status.block.is_final = true;
            // eprintln!(
            //     "Calling close stage from premature termination {stage_index} {}",
            //     self.stages[stage_index].lock().unwrap()
            // );
            self.close_stages(stage_index);
        }
        // but unless the stage said 'no more blocks' *previously*, we still process this one.
        if was_already_closed {
            self.release_block(block_status.block.initial_molecule_count());
        } else if block_status.current_stage >= self.stages.len() {
            // eprintln!("outputing {}", block_status.block_no);
            self.output_block(block_status)?;
            // Block completed all stages - will be sent to output
            // Keep it in active_blocks so find_completed_blocks can find it
        } else {
            self.queue_block(block_status)?;
        }

        self.queue_stalled()?;
        Ok(())
    }

    fn queue_stalled(&mut self) -> Result<()> {
        let mut new_stalled = Vec::new();
        for block_status in self
            .stalled_blocks
            .take()
            .expect("Should aways be some at this point")
        {
            match Self::stage_can_take_block(
                &self.stage_progress,
                block_status.current_stage,
                block_status.block.block_no(),
            ) {
                CanTake::No => new_stalled.push(block_status),
                CanTake::Yes => {
                    self.send_block_to_workers(block_status)?;
                }
                CanTake::Drop => {
                    // eprintln!(
                    //     "Dropping stalled block {} (next stage was {}",
                    //     block_status.block_no, block_status.current_stage
                    // );
                    self.release_block(block_status.block.initial_molecule_count()); // we drop it here.
                }
            }
        }
        self.stalled_blocks = Some(new_stalled);
        Ok(())
    }

    fn output_block(&mut self, block_status: BlockStatus) -> Result<()> {
        let block_no = block_status.block.block_no();
        if self
            .output_tx
            .send((block_status.block, block_status.expected_read_count))
            .is_err()
        {
            //cov:excl-start
            bail!("Failed to send completed block {block_no} to output");
            // cov:excl-stop
        }
        self.queue_stalled()
    }

    pub fn close_stages(&mut self, from_stage_index: usize) {
        self.stage_progress[from_stage_index].closed = true;
        for stage_index in (0..from_stage_index).rev() {
            if self.stage_progress[stage_index].transmits_premature_termination {
                self.stage_progress[stage_index].closed = true;
            } else {
                break;
            }
        }
    }

    pub fn finalize_reports(&mut self, demultiplex_infos: &[(usize, OptDemultiplex)]) {
        for (stage_index, stage) in self.stages.iter().enumerate() {
            // Find appropriate demultiplex info for this stage
            let mut demultiplex_info = &OptDemultiplex::No;
            for (idx, info) in demultiplex_infos.iter().rev() {
                if *idx <= stage_index {
                    demultiplex_info = info;
                    break;
                }
            }

            match stage.transformation.finalize(demultiplex_info) {
                Ok(Some(mut report)) => {
                    if matches!(demultiplex_info, OptDemultiplex::Yes(_)) {
                        let inner = std::mem::replace(
                            &mut report.contents,
                            serde_json::Value::Object(serde_json::Map::new()),
                        );
                        report
                            .contents
                            .as_object_mut()
                            .expect("just created")
                            .insert("multiplexed".to_string(), inner);
                    }
                    if let Ok(mut collector) = self.report_collector.lock() {
                        collector.push(report);
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    self.error_collector
                        .lock()
                        .expect("error collector poisened")
                        .push(format!("Error finalizing report: {err:?}"));
                }
            }
        }

        {
            let reports = self
                .report_collector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for stage in self.stages.iter() {
                if let Err(err) = stage.transformation.post_finalize(&reports) {
                    self.error_collector
                        .lock()
                        .expect("error collector poisened")
                        .push(format!("Error in post_finalize: {err:?}"));
                }
            }
        }
    }
}

pub fn worker_thread(
    _worker_id: usize,
    todo_rx: &Receiver<WorkItem>,
    done_tx: &Sender<WorkResult>,
    stages: &[Arc<Stage>],
    input_info: &transformations::InputInfo,
    demultiplex_infos: &[(usize, OptDemultiplex)],
) {
    while let Ok(work_item) = todo_rx.recv() {
        let result = process_work_item(work_item, stages, input_info, demultiplex_infos);

        if done_tx.send(result).is_err() {
            break; // Coordinator shut down
        }
    }
}

fn process_work_item(
    mut work_item: WorkItem,
    stages: &[Arc<Stage>],
    input_info: &transformations::InputInfo,
    demultiplex_infos: &[(usize, OptDemultiplex)],
) -> WorkResult {
    use itertools::Itertools;
    let stage_index = work_item.stage_index;

    // Find appropriate demultiplex info
    let mut demultiplex_info = &OptDemultiplex::No;
    for (idx, info) in demultiplex_infos.iter().rev() {
        if *idx <= stage_index {
            demultiplex_info = info;
            break;
        }
    }

    let block_no = work_item.block.block_no();
    let expected_read_count = work_item.expected_read_count;
    let stage = &stages[stage_index];

    //now calculate virtual tags.
    for tag in &stage.allowed_tags {
        match tag {
            TagLabel::Length(segment_index, _) => {
                let read_lengths = {
                    match segment_index {
                        fastqrab_steps::config::SegmentIndexOrAll::All => {
                            let mut read_lengths = vec![0; work_item.block.segments[0].len()];
                            for segment in &work_item.block.segments {
                                for (ii, read_len) in segment.seq_quals.iter_seq_lens().enumerate()
                                {
                                    read_lengths[ii] += read_len;
                                }
                            }
                            read_lengths
                        }
                        fastqrab_steps::config::SegmentIndexOrAll::Indexed(index) => {
                            work_item.block.segments[index.as_index()]
                                .seq_quals
                                .iter_seq_lens()
                                .collect()
                        }
                    }
                };
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "Unlikely to exceed f64 precise regions"
                )]
                let read_lengths: Vec<f64> = read_lengths.into_iter().map(|x| x as f64).collect();
                work_item
                    .block
                    .tags
                    .insert(tag.clone(), TagColumn::Numeric(read_lengths));
            }
            TagLabel::Normal(_) => {}
            TagLabel::TagLength(tag_name, _) => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "Unlikely to exceed f64 precise regions"
                )]
                let tag_lengths: Vec<f64> = match work_item
                    .block
                    .tags
                    .get(&TagLabel::Normal(tag_name.clone()))
                    .expect("Tag not present. Should have been caught in validation. Bug")
                {
                    TagColumn::Location(col) => {
                        col.iter_row_lengths(None).map(|n| n as f64).collect()
                    }
                    TagColumn::String(items) => items
                        .iter()
                        .map(|opt_str| opt_str.as_ref().map_or(0, |s| s.len()))
                        .map(|n| n as f64)
                        .collect(),
                    TagColumn::Numeric(_) => unreachable!(
                        "len of a numeric tag not defined. Should have been caught in validation"
                    ), // cov:excl-line
                    TagColumn::Bool(_) => unreachable!(
                        "len of a bool tag not defined. Should have been caught in validation"
                    ), // cov:excl-line
                };
                work_item
                    .block
                    .tags
                    .insert(tag.clone(), TagColumn::Numeric(tag_lengths));
            }
            TagLabel::TagLocation {
                source,
                definition: _,
            } => {
                // The *current* location: the tag's captured (read-relative)
                // regions lifted through every edit the source segment has seen
                // since the tag was born, into the segment's present frame. A
                // region that was cut away (or split) by an intervening edit has
                // no contiguous image and is reported as lost (an empty cell);
                // `initial_location_<tag>` still shows where it started.
                let tag_locations: StringColumn = {
                    let col = work_item
                        .block
                        .tags
                        .get(&TagLabel::Normal(source.clone()))
                        .expect("Tag not present. Should have been caught in validation. Bug")
                        .as_locations()
                        .expect("Tag was not location. should have been cought in validation. Bug");
                    let segment_index = col.source_id() as usize;
                    let segment_name = &input_info.segment_order[segment_index];
                    let segment = &work_item.block.segments[segment_index];
                    (0..col.row_count())
                        .map(|row| {
                            // Lift each captured (birth-frame) region forward into
                            // the segment's *current* frame, replaying only the
                            // edits applied since the tag was born.
                            let (born_gen, born_len) = col.row_born(row);
                            let view = segment.seq_quals.ops_since(born_gen, row).expect(
                                "born generation captured from this pod; row in range. Bug",
                            );
                            let mut lifted: Vec<(usize, usize)> = Vec::new();
                            for (start, len) in col.row_regions(row) {
                                let (start, len) = (start as usize, len as usize);
                                match view.map_region(start, len, born_len) {
                                    Ok(RegionLift::Kept { start, len }) => {
                                        lifted.push((start, len));
                                    }
                                    // Cut away or split by an intervening edit.
                                    Ok(RegionLift::Dropped) | Err(_) => {}
                                }
                            }
                            if lifted.is_empty() {
                                None
                            } else {
                                let mut seq = BString::new(segment_name.as_bytes().to_vec());
                                seq.push(b':');
                                let mut first = true;
                                for (start, len) in &lifted {
                                    if !first {
                                        seq.push(b',');
                                    }
                                    first = false;
                                    seq.extend_from_slice(
                                        format!("{}-{}", start, start + len).as_bytes(),
                                    );
                                }
                                Some(seq)
                            }
                        })
                        .collect()
                };
                work_item
                    .block
                    .tags
                    .insert(tag.clone(), TagColumn::String(tag_locations));
            }
            TagLabel::TagInitialLocation {
                source,
                definition: _,
            } => {
                let tag_locations: StringColumn = {
                    let col = work_item
                        .block
                        .tags
                        .get(&TagLabel::Normal(source.clone()))
                        .expect("Tag not present. Should have been caught in validation. Bug")
                        .as_locations()
                        .expect("Tag was not location. should have been cought in validation. Bug");
                    let segment_name = &input_info.segment_order[col.source_id() as usize];
                    col.iter_row_regions()
                        .map(|start_lens| {
                            if start_lens.is_empty() {
                                None
                            } else {
                                let mut seq = BString::new(segment_name.as_bytes().to_vec());
                                seq.push(b':');
                                let mut first = true;
                                for (start, len) in start_lens.iter() {
                                    if !first {
                                        seq.push(b',');
                                    }
                                    first = false;
                                    seq.extend_from_slice(
                                        format!("{}-{}", start, start + len).as_bytes(),
                                    );
                                }
                                Some(seq)
                            }
                        })
                        .collect()
                };
                work_item
                    .block
                    .tags
                    .insert(tag.clone(), TagColumn::String(tag_locations));
            }
            TagLabel::ReadNo => {
                let start = work_item.block.first_read_sequential_number;
                let end = work_item.block.first_read_sequential_number + work_item.block.len();
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "Unlikely to exceed f64 precise regions"
                )]
                let read_nos: Vec<f64> = (start..end).map(|x| x as f64).collect();
                work_item
                    .block
                    .tags
                    .insert(tag.clone(), TagColumn::Numeric(read_nos));
            }
        }
    }
    // Drop tags this stage declared it forgets (ForgetTag / ForgetAllTags /
    // conditional Swap / MergeReads). Removing them here — before the step's
    // `apply` — means the step never sees them and cannot re-add them, so each
    // forgetting step no longer has to do this by hand. Done before the
    // `unused_tags` split so they are neither passed to the step nor restored
    // afterward.
    for tag in &stage.forgotten_tags {
        work_item.block.tags.shift_remove(tag);
    }

    let unused_tags: Vec<_> = work_item
        .block
        .tags
        .extract_if(.., |k, _v| !stage.allowed_tags.contains(k))
        .collect();

    let first_read_sequential_number = work_item.block.first_read_sequential_number;

    let result = {
        let mut input_info = input_info.clone();
        input_info.initial_filter_capacity = expected_read_count;

        let len_before = work_item.block.len();

        let block_tag_count = work_item.block.tags.len();
        let result = stage
            .transformation
            .apply(work_item.block, &input_info, demultiplex_info);

        if let Ok(ref result) = result {
            let len_after = result.0.len();
            if len_before != len_after {
                // mutants false positve.
                // defensive construct against coding errors
                //cov:excl-start
                assert!(
                    stage.allowed_tags.len() == block_tag_count,
                    "A filtering stage forgot to declare must_see_all_tags=true: {:?}. Declared {} tags, block had {} tags",
                    stage.transformation,
                    stage.allowed_tags.len(),
                    block_tag_count
                );
                //cov:excl-stop
            }
        }
        result
    };

    match result {
        Ok((mut result_block, stage_continue)) => {
            result_block.tags.extend(unused_tags);
            for tag in &stage.allowed_tags {
                if tag.is_virtual() {
                    result_block.tags.swap_remove(tag);
                }
            }
            //make sure all tags have the same length
            let all_tag_lengths_equal = result_block.tags.values().map(TagColumn::len).all_equal();
            // cov:excl-start
            if !all_tag_lengths_equal {
                let tags_and_lengths: String = result_block
                    .tags
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.len()))
                    .join(", ");
                panic!(
                    "Unequal tag lengths after stage {:?}:. Tags & lengths: {tags_and_lengths}. This is a bug!. \n\
                    Best case it needs to declare must_see_all_tags=true in TagUser::get_tag_usage()",
                    stage.transformation
                );
            }
            // assert!(
            //     all_tag_lengths_equal,
            //     "Unequal tag lengths after stage {:?}:. Tags: {:?}. This is a bug!. \n\
            //     Best case it needs to declare must_see_all_tags=true in TagUser::get_tag_usage()",
            //     stage.transformation, result_block.tags
            // );
            // cov:excl-stop
            if let Some(tag_len) = result_block.tags.values().next().map(TagColumn::len) {
                //cov:excl-start
                assert!(
                    result_block.len() == tag_len,
                    "Tag lengths don't match block length after stage {:?}:. Block len: {}. Tag len: {tag_len} This is a bug!. \n\
                    Best case it needs to declare must_see_all_tags=true in TagUser::get_tag_usage()",
                    stage.transformation,
                    result_block.len(),
                );

                //cov:excl-stop
            }
            WorkResult {
                work_item: WorkItem {
                    block: result_block,
                    expected_read_count,
                    stage_index,
                },
                stage_continue,
                error: None,
            }
        }
        Err(e) => WorkResult {
            work_item: WorkItem {
                block: io::FastQBlocksCombined::new(
                    vec![FastQChunk::new_empty()],
                    None,
                    Default::default(),
                    false,
                    block_no,
                    first_read_sequential_number,
                ),
                expected_read_count,
                stage_index,
            },
            stage_continue: false,
            error: Some(e),
        },
    }
}
