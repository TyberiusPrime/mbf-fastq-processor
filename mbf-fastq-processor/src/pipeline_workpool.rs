use anyhow::Result;
use crossbeam::channel::{Receiver, Sender, select};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    pub block: Option<io::FastQBlocksCombined>,
    pub expected_read_count: Option<usize>,
}

pub struct StageProgress {
    pub highest_completed_block: usize,
    pub needs_serial: bool,
}

pub struct WorkResult {
    pub work_item: WorkItem,
    pub result_block: io::FastQBlocksCombined,
    pub stage_continue: bool,
    pub error: Option<anyhow::Error>,
}

pub struct WorkpoolCoordinator {
    stages: Vec<Arc<Mutex<Transformation>>>,
    stage_progress: Vec<StageProgress>,
    stalled_blocks: Vec<BlockStatus>,
    max_blocks_in_flight: usize,
    current_blocks_in_flight: usize,
    active_blocks: HashMap<usize, BlockStatus>, // block_no -> BlockStatus
    pending_work_items: usize, // Number of work items sent to workers but not yet completed
    pub error_collector: Arc<Mutex<Vec<String>>>,
}

impl WorkpoolCoordinator {
    pub fn new(
        stages: Vec<Transformation>,
        max_blocks: usize,
        error_collector: Arc<Mutex<Vec<String>>>,
    ) -> (Self, Vec<Arc<Mutex<Transformation>>>) {
        let stage_progress: Vec<StageProgress> = stages
            .iter()
            .map(|stage| StageProgress {
                highest_completed_block: 0,
                needs_serial: stage.needs_serial(),
            })
            .collect();

        let arc_stages: Vec<Arc<Mutex<Transformation>>> = stages
            .into_iter()
            .map(|stage| Arc::new(Mutex::new(stage)))
            .collect();

        let stages_for_workers = arc_stages.clone();

        let coordinator = Self {
            stages: arc_stages,
            stage_progress,
            stalled_blocks: Vec::new(),
            max_blocks_in_flight: max_blocks,
            current_blocks_in_flight: 0,
            active_blocks: HashMap::new(),
            pending_work_items: 0,
            error_collector,
        };

        (coordinator, stages_for_workers)
    }

    pub fn process_incoming_block(
        &mut self,
        block_no: usize,
        block: io::FastQBlocksCombined,
        expected_read_count: Option<usize>,
    ) {
        // Check if we're at capacity
        if self.current_blocks_in_flight >= self.max_blocks_in_flight {
            return;
        }

        let block_status = BlockStatus {
            block_no,
            current_stage: 0,
            block: Some(block),
            expected_read_count,
        };

        self.current_blocks_in_flight += 1;
        self.active_blocks.insert(block_no, block_status);
    }

    pub fn process_completed_work(&mut self, work_result: WorkResult) -> bool {
        let block_no = work_result.work_item.block_no;
        let stage_index = work_result.work_item.stage_index;

        self.pending_work_items -= 1;

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
            return false;
        }

        // Create or update block status
        let result_block = work_result.result_block;
        let mut block_status = BlockStatus {
            block_no,
            current_stage: stage_index + 1,
            block: Some(result_block),
            expected_read_count: work_result.work_item.expected_read_count,
        };

        if !work_result.stage_continue {
            // Stage requested premature termination - mark block as final
            if let Some(ref mut block) = block_status.block {
                block.is_final = true;
            }
        }

        if block_status.current_stage >= self.stages.len() {
            // Block completed all stages - will be sent to output
            self.current_blocks_in_flight -= 1;
            // Keep it in active_blocks so find_completed_blocks can find it
            self.active_blocks.insert(block_no, block_status);
        } else {
            // Put block back into active_blocks for next stage
            self.active_blocks.insert(block_no, block_status);
        }
        true
    }

    pub fn find_ready_work(&mut self) -> Vec<WorkItem> {
        let mut ready_work = Vec::new();

        // Check active blocks for ready work
        let active_blocks: Vec<_> = self.active_blocks.values().cloned().collect();

        for block_status in active_blocks {
            // Block progress through stages
            if block_status.current_stage < self.stages.len() {
                if self
                    .can_schedule_block_for_stage(block_status.block_no, block_status.current_stage)
                {
                    if let Some(block) = block_status.block.clone() {
                        let work_item = WorkItem {
                            block_no: block_status.block_no,
                            block,
                            expected_read_count: block_status.expected_read_count,
                            stage_index: block_status.current_stage,
                        };
                        ready_work.push(work_item);
                        self.pending_work_items += 1;
                        // Remove from active blocks since we're sending it to workers
                        self.active_blocks.remove(&block_status.block_no);
                    }
                } else {
                    // Block cannot be scheduled due to serial constraint
                }
            }
        }

        // Also check stalled blocks
        let stalled_blocks = std::mem::take(&mut self.stalled_blocks);
        for block_status in stalled_blocks {
            if block_status.current_stage < self.stages.len() {
                if self
                    .can_schedule_block_for_stage(block_status.block_no, block_status.current_stage)
                {
                    if let Some(block) = block_status.block {
                        let work_item = WorkItem {
                            block_no: block_status.block_no,
                            block,
                            expected_read_count: block_status.expected_read_count,
                            stage_index: block_status.current_stage,
                        };
                        ready_work.push(work_item);
                        self.pending_work_items += 1;
                    }
                } else {
                    // Block is still stalled
                    self.stalled_blocks.push(block_status);
                }
            }
        }

        ready_work
    }

    pub fn can_schedule_block_for_stage(&self, block_no: usize, stage_index: usize) -> bool {
        let stage_progress = &self.stage_progress[stage_index];

        if stage_progress.needs_serial {
            // For serial stages, can only process block N if block N-1 is complete
            block_no == stage_progress.highest_completed_block + 1
        } else {
            // Parallel stages can process any block
            true
        }
    }

    pub fn find_completed_blocks(
        &mut self,
    ) -> Vec<(usize, io::FastQBlocksCombined, Option<usize>)> {
        let mut completed = Vec::new();
        let mut completed_block_nos = Vec::new();

        for (block_no, block_status) in &self.active_blocks {
            if block_status.current_stage >= self.stages.len() {
                if let Some(ref block) = block_status.block {
                    completed.push((*block_no, block.clone(), block_status.expected_read_count));
                    completed_block_nos.push(*block_no);
                }
            }
        }

        // Remove completed blocks from active_blocks
        for block_no in completed_block_nos {
            self.active_blocks.remove(&block_no);
        }

        completed
    }

    pub fn finalize_reports(
        &mut self,
        report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
        demultiplex_infos: &[(usize, OptDemultiplex)],
    ) {
        for (stage_index, stage) in self.stages.iter().enumerate() {
            if let Ok(mut stage_locked) = stage.lock() {
                // Find appropriate demultiplex info for this stage
                let mut demultiplex_info = &OptDemultiplex::No;
                for (idx, info) in demultiplex_infos.iter().rev() {
                    if *idx <= stage_index {
                        demultiplex_info = info;
                        break;
                    }
                }

                match stage_locked.finalize(demultiplex_info) {
                    Ok(Some(report)) => {
                        if let Ok(mut collector) = report_collector.lock() {
                            collector.push(report);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("Error finalizing report: {:?}", err);
                    }
                }
            }
        }
    }
}

pub fn run_coordinator(
    mut coordinator: WorkpoolCoordinator,
    incoming_rx: Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>,
    todo_tx: Sender<WorkItem>,
    done_rx: Receiver<WorkResult>,
    output_tx: Sender<(usize, io::FastQBlocksCombined, Option<usize>)>,
    report_collector: Arc<Mutex<Vec<transformations::FinalizeReportResult>>>,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
) {
    let mut input_closed = false;

    loop {
        if input_closed {
            // Only listen for completed work when input is closed
            match done_rx.recv() {
                Ok(work_result) => {
                    if !coordinator.process_completed_work(work_result) {
                        break; // Coordinator decided to terminate
                    }
                }
                Err(_) => {
                    break; // Workers closed
                }
            }
        } else {
            // Listen for both incoming and done messages
            select! {
                recv(incoming_rx) -> msg => {

                    match msg {
                        Ok((block_no, block, expected_read_count)) => {
                            // if block.is_final {
                            //
                            //     final_block_seen = true;
                            //     coordinator.handle_final_block(block_no, &output_tx);
                            //     // Don't break yet - wait for active work to complete
                            // } else {
                                coordinator.process_incoming_block(block_no, block, expected_read_count);
                            //}
                        }
                        Err(_) => {

                            input_closed = true;
                            if coordinator.pending_work_items == 0 {
                                break; // Input closed and no pending work
                            }
                            // Continue processing to handle remaining work
                        }
                    }
                }
                recv(done_rx) -> msg => {

                    match msg {
                        Ok(work_result) => {
                            coordinator.process_completed_work(work_result);
                        }
                        Err(_) => {

                            break; // Workers closed
                        }
                    }
                }
            }
        }

        // Check for ready work after each event
        let ready_work = coordinator.find_ready_work();
        for work_item in ready_work {
            if todo_tx.send(work_item).is_err() {
                // Workers have shut down
                break;
            }
        }

        // Send completed blocks to output
        let completed_blocks = coordinator.find_completed_blocks();
        for (block_no, block, expected_read_count) in completed_blocks {
            if output_tx
                .send((block_no, block, expected_read_count))
                .is_err()
            {
                // Output thread has shut down
                break;
            }
        }

        // Check if we should terminate
        if input_closed
            && coordinator.active_blocks.is_empty()
            && coordinator.stalled_blocks.is_empty()
            && coordinator.pending_work_items == 0
        {
            break;
        }
    }

    // Finalize reports before ending
    coordinator.finalize_reports(report_collector, &demultiplex_infos);
}

pub fn worker_thread(
    _worker_id: usize,
    todo_rx: Receiver<WorkItem>,
    done_tx: Sender<WorkResult>,
    stages: Vec<Arc<Mutex<Transformation>>>,
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
    stages: &[Arc<Mutex<Transformation>>],
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
        let mut stage = stages[stage_index]
            .lock()
            .expect("stage mutex should not be poisoned");

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
                block: result_block.clone(),
                expected_read_count,
                stage_index,
            },
            result_block,
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
            result_block: io::FastQBlocksCombined {
                segments: vec![io::FastQBlock::empty()],
                output_tags: None,
                tags: Default::default(),
                is_final: false,
            },
            stage_continue: false,
            error: Some(e),
        },
    }
}
