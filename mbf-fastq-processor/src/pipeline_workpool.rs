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
    demultiplex::OptDemultiplex,
    io,
    transformations::{self, Step, Transformation},
};

pub struct WorkItem {
    pub block_no: usize,
    pub block: io::FastQBlocksCombined,
    pub expected_read_count: Option<usize>,
    pub stage_index: usize,
}

#[derive(Clone)]
pub struct BlockStatus {
    pub block_no: usize,
    pub current_stage: usize,
    pub block: io::FastQBlocksCombined,
    pub expected_read_count: Option<usize>,
}

impl std::fmt::Debug for BlockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockStatus")
            .field("block_no", &self.block_no)
            .field("current_stage", &self.current_stage)
            .finish()
    }
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
    stages: Vec<Arc<Transformation>>,
    stage_progress: Vec<StageProgress>,

    stalled_blocks: Option<Vec<BlockStatus>>, //blocks waiting to get ready.

    current_blocks_in_flight: usize, // that's 'within pipeline, stalled + currently being worked on.
    max_blocks_in_flight: usize,
    pending_work_items: usize, // Number of work items sent to workers but not yet completed

    incoming_rx: Option<Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>>,
    todo_tx: Sender<WorkItem>,     //towards workers
    done_rx: Receiver<WorkResult>, //back from workers
    output_tx: Sender<(usize, io::FastQBlocksCombined, Option<usize>)>,

    report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
}

enum CanTake {
    Yes,
    No,
    Drop,
}

impl WorkpoolCoordinator {
    pub fn new(
        stages: Vec<Transformation>,
        max_blocks_in_flight: usize,
        incoming_rx: Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>,
        todo_tx: Sender<WorkItem>,     //towards workers
        done_rx: Receiver<WorkResult>, //back from workers
        output_tx: Sender<(usize, io::FastQBlocksCombined, Option<usize>)>,

        report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
        error_collector: Arc<Mutex<Vec<String>>>,
    ) -> (Self, Vec<Arc<Transformation>>) {
        let stage_progress: Vec<StageProgress> = stages
            .iter()
            .map(|stage| StageProgress {
                highest_completed_block: 0,
                needs_serial: stage.needs_serial(),
                closed: false,
                transmits_premature_termination: stage.transmits_premature_termination(),
            })
            .collect();

        let arc_stages: Vec<Arc<Transformation>> =
            stages.into_iter().map(|stage| Arc::new(stage)).collect();

        let stages_for_workers = arc_stages.clone();

        let coordinator = Self {
            stages: arc_stages,
            stage_progress,
            stalled_blocks: Some(Vec::new()),
            max_blocks_in_flight: max_blocks_in_flight,
            current_blocks_in_flight: 0,
            pending_work_items: 0,

            incoming_rx: Some(incoming_rx),
            todo_tx,
            done_rx,
            output_tx,

            error_collector,
            report_collector,
        };

        (coordinator, stages_for_workers)
    }

    pub fn run(&mut self, demultiplex_infos: &[(usize, OptDemultiplex)]) {
        loop {
            // Check if we're at capacity
            let accept_new_incoming = self.current_blocks_in_flight < self.max_blocks_in_flight;
            if self.incoming_rx.is_none() || !accept_new_incoming {
                // Only listen for completed work when input is closed
                match self.done_rx.recv() {
                    //match done_rx.recv_timeout(std::time::Duration::from_millis(1000)) {
                    Ok(work_result) => {
                        if self.process_completed_work(work_result).is_err() {
                            break; // Coordinator decided to terminate because of an error.
                        }
                    }
                    Err(_) => {
                        self
                        .error_collector
                        .lock()
                        .expect("error collector mutex poisoned")
                        .push(
                            "No incoming blocks and no completed work; terminating coordinator."
                                .to_string(),
                        );
                        break; // WoSleep::rkers closed
                    }
                }
            } else {
                // Listen for both incoming and done messages
                select! {
                    recv(self.incoming_rx.as_ref().unwrap()) -> msg => {

                        match msg {
                            Ok((block_no, block, expected_read_count)) => {
                                if self.process_incoming_block(block_no, block, expected_read_count).is_err() {
                                    break
                                };
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
                                    break; // Coordinator decided to terminate because of an error.
                                };
                            }
                            Err(_) => {
                                break; // Workers pipe crashed?
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
                && self.stalled_blocks.as_ref().unwrap().is_empty()
                && self.pending_work_items == 0
            {
                break;
            }
        }

        // Finalize reports before ending
        self.finalize_reports(&demultiplex_infos);
    }

    pub fn process_incoming_block(
        &mut self,
        block_no: usize,
        block: io::FastQBlocksCombined,
        expected_read_count: Option<usize>,
    ) -> Result<()> {
        // eprintln!("Adding to pipeline: {block_no}");
        let block_status = BlockStatus {
            block_no,
            current_stage: 0,
            block: block,
            expected_read_count,
        };
        self.current_blocks_in_flight += 1;
        self.queue_block(block_status)?;
        Ok(())
    }

    fn queue_block(&mut self, block_status: BlockStatus) -> Result<()> {
        if self.stages.is_empty() {
            self.output_block(block_status)?;
        } else {
            match Self::stage_can_take_block(
                &self.stage_progress,
                block_status.current_stage,
                block_status.block_no,
            ) {
                CanTake::Yes => {
                    // eprintln!("Sending block {} off to process stage {}", block_status.block_no, block_status.current_stage);
                    self.send_block_to_workers(block_status)?;
                }
                CanTake::No => {
                    self.stalled_blocks.as_mut().unwrap().push(block_status);
                }
                CanTake::Drop => {
                    // eprintln!(
                    //     "Dropping after stage: block {} (next stage was {}",
                    //     block_status.block_no, block_status.current_stage
                    // );
                    self.current_blocks_in_flight -= 1; // we drop it here
                }
            }
        }
        Ok(())
    }

    fn stage_can_take_block(
        stage_progress: &Vec<StageProgress>,
        stage_index: usize,
        block_no: usize,
    ) -> CanTake {
        if stage_progress[stage_index].closed {
            CanTake::Drop
        } else if !stage_progress[stage_index].needs_serial {
            CanTake::Yes
        } else if stage_progress[stage_index].highest_completed_block + 1 == block_no {
            CanTake::Yes
        } else {
            CanTake::No
        }
    }

    pub fn send_block_to_workers(&mut self, block_status: BlockStatus) -> Result<()> {
        let work_item = WorkItem {
            block_no: block_status.block_no,
            block: block_status.block,
            expected_read_count: block_status.expected_read_count,
            stage_index: block_status.current_stage,
        };
        if self.todo_tx.send(work_item).is_ok() {
            self.pending_work_items += 1;
            Ok(())
        } else {
            bail!(
                "Failed to send work item for block {}",
                block_status.block_no
            );
        }
    }

    pub fn process_completed_work(&mut self, work_result: WorkResult) -> Result<()> {
        let block_no = work_result.work_item.block_no;
        let stage_index = work_result.work_item.stage_index;

        self.pending_work_items -= 1;
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
                .expect("error collector mutex poisoned")
                .push(format!("Error in stage {}: {:?}", stage_index, error));
            bail!("error detected");
        }

        // Create or update block status
        let mut block_status = BlockStatus {
            block_no,
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
            self.current_blocks_in_flight -= 1;
        } else {
            if block_status.current_stage >= self.stages.len() {
                // eprintln!("outputing {}", block_status.block_no);
                self.output_block(block_status)?;
                // Block completed all stages - will be sent to output
                // Keep it in active_blocks so find_completed_blocks can find it
            } else {
                self.queue_block(block_status)?;
            }
        }
        self.queue_stalled()?;
        Ok(())
    }

    fn queue_stalled(&mut self) -> Result<()> {
        let mut new_stalled = Vec::new();
        for block_status in self.stalled_blocks.take().unwrap() {
            match Self::stage_can_take_block(
                &self.stage_progress,
                block_status.current_stage,
                block_status.block_no,
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
                    self.current_blocks_in_flight -= 1; // we drop it here
                }
            }
        }
        self.stalled_blocks = Some(new_stalled);
        Ok(())
    }

    fn output_block(&mut self, block_status: BlockStatus)-> Result<()> {
        self.current_blocks_in_flight -= 1;
        if self
            .output_tx
            .send((
                block_status.block_no,
                block_status.block,
                block_status.expected_read_count,
            ))
            .is_err()
        {
            // eprintln!(
            //     "Failed to send completed block {} to output",
            //     block_status.block_no
            // );
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

            match stage.finalize(demultiplex_info) {
                Ok(Some(report)) => {
                    if let Ok(mut collector) = self.report_collector.lock() {
                        collector.push(report);
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    self.error_collector
                        .lock()
                        .expect("error collector poisened")
                        .push(format!("Error finalizing report: {:?}", err));
                }
            }
        }
    }
}

pub fn worker_thread(
    _worker_id: usize,
    todo_rx: Receiver<WorkItem>,
    done_tx: Sender<WorkResult>,
    stages: Vec<Arc<Transformation>>,
    input_info: transformations::InputInfo,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
) -> Result<()> {
    while let Ok(work_item) = todo_rx.recv() {
        let result = process_work_item(
            work_item,
            &stages,
            &input_info,
            &demultiplex_infos,
            &timing_collector,
        );

        if done_tx.send(result).is_err() {
            break; // Coordinator shut down
        }
    }

    Ok(())
}

fn process_work_item(
    work_item: WorkItem,
    stages: &[Arc<Transformation>],
    input_info: &transformations::InputInfo,
    demultiplex_infos: &[(usize, OptDemultiplex)],
    timing_collector: &Arc<Mutex<Vec<crate::timing::StepTiming>>>,
) -> WorkResult {
    let stage_index = work_item.stage_index;

    // Find appropriate demultiplex info
    let mut demultiplex_info = &OptDemultiplex::No;
    for (idx, info) in demultiplex_infos.iter().rev() {
        if *idx <= stage_index {
            demultiplex_info = info;
            break;
        }
    }

    // Execute the transformation with timing
    let (wall_start, cpu_start) = crate::timing::StepTiming::start();

    let block_no = work_item.block_no;
    let expected_read_count = work_item.expected_read_count;

    let (result, stage_name) = {
        let stage = &stages[stage_index];

        let mut input_info = input_info.clone();
        input_info.initial_filter_capacity = expected_read_count;

        (
            stage.apply(work_item.block, &input_info, block_no, demultiplex_info),
            format!("{}", stage),
        )
    };

    let timing =
        crate::timing::StepTiming::from_start(stage_index, stage_name, wall_start, cpu_start);

    timing_collector
        .lock()
        .expect("timing collector mutex should not be poisoned")
        .push(timing);

    match result {
        Ok((result_block, stage_continue)) => WorkResult {
            work_item: WorkItem {
                block_no,
                block: result_block,
                expected_read_count,
                stage_index,
            },
            stage_continue,
            error: None,
        },
        Err(e) => WorkResult {
            work_item: WorkItem {
                block_no,
                block: io::FastQBlocksCombined {
                    segments: vec![io::FastQBlock::empty()],
                    output_tags: None,
                    tags: Default::default(),
                    is_final: false,
                },
                expected_read_count,
                stage_index,
            },
            stage_continue: false,
            error: Some(e),
        },
    }
}
