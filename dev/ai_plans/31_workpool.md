# Workpool Pipeline Implementation Plan

## Overview

Replace the current oversubscribing, unbalanced pipeline with a workpool-based alternative that provides better resource control and configurable parallelism limits. This will be implemented behind a `--workpool` CLI flag for both `process` and `verify` commands.

## Current Architecture Analysis

The existing pipeline in `mbf-fastq-processor/src/pipeline.rs` uses:
- Multiple input threads (one per segment)
- A combiner thread that matches blocks across segments
- Per-stage threads with unlimited parallelism (except `needs_serial` stages)
- One output thread with ordering enforcement

Key characteristics:
- Stages spawn `thread_count` threads each (except `needs_serial` stages which use 1 thread)
- Can lead to thread oversubscription: N_stages × thread_count threads
- No global resource management across stages
- Ordering enforced through buffering in serial stages and output thread

## Workpool Architecture Design

### Core Components

1. **Coordinator Thread**
   - Tracks highest `block_no` processed by each stage
   - Maintains list of stalled blocks waiting for their turn
   - Polls `done_rx` (completed work) and `incoming_rx` (new blocks from combiner)
   - Enforces ordering for `needs_serial` stages
   - Manages global block count limits
   - Sends work to `todo_tx` when blocks are ready

2. **Worker Pool**
   - Fixed number of worker threads (configurable)
   - Pull work from `todo_tx` channel
   - Execute stage transformations
   - Send results to `done_tx` channel
   - Workers are stage-agnostic and can handle any transformation

3. **Block Flow Management**
   - `incoming_tx/rx`: Combiner → Coordinator (new blocks)
   - `todo_tx/rx`: Coordinator → Workers (work assignments)
   - `done_tx/rx`: Workers → Coordinator (completed work)
   - `output_tx/rx`: Coordinator → Output thread (final results)

## Detailed Implementation Plan

### Phase 1: Core Workpool Infrastructure

#### 1.1 Create Workpool Data Structures
**File**: `mbf-fastq-processor/src/pipeline_workpool.rs` (new)

```rust
pub struct WorkItem {
    block_no: usize,
    block: io::FastQBlocksCombined,
    expected_read_count: Option<usize>,
    stage_index: usize,
}

pub struct BlockStatus {
    block_no: usize,
    current_stage: usize,
    block: Option<io::FastQBlocksCombined>,
    expected_read_count: Option<usize>,
}

pub struct StageProgress {
    highest_completed_block: usize,
    needs_serial: bool,
}

pub struct WorkpoolCoordinator {
    stages: Vec<Transformation>,
    stage_progress: Vec<StageProgress>,
    stalled_blocks: Vec<BlockStatus>,
    max_blocks_in_flight: usize,
    current_blocks_in_flight: usize,
}
```

#### 1.2 Implement Coordinator Logic
**File**: `mbf-fastq-processor/src/workpool.rs`

```rust
impl WorkpoolCoordinator {
    fn new(stages: Vec<Transformation>, max_blocks: usize) -> Self;
    fn process_incoming_block(&mut self, block_no: usize, block: io::FastQBlocksCombined, expected_read_count: Option<usize>);
    fn process_completed_work(&mut self, work_item: WorkItem, result: WorkResult);
    fn find_ready_work(&mut self) -> Vec<WorkItem>;
    fn can_schedule_block_for_stage(&self, block_no: usize, stage_index: usize) -> bool;
    fn advance_block_to_next_stage(&mut self, mut block_status: BlockStatus);
}
```

Key coordinator logic:
- Track `highest_completed_block` per stage
- For `needs_serial` stages: only schedule block N if block N-1 is complete
- For parallel stages: schedule any block regardless of order
- Maintain global block count limit
- Move blocks through stages sequentially

#### 1.3 Implement Worker Pool
**File**: `mbf-fastq-processor/src/workpool.rs`

```rust
pub struct WorkResult {
    work_item: WorkItem,
    result_block: io::FastQBlocksCombined,
    stage_continue: bool,
    error: Option<anyhow::Error>,
}

fn worker_thread(
    worker_id: usize,
    todo_rx: Receiver<WorkItem>,
    done_tx: Sender<WorkResult>,
    input_info: transformations::InputInfo,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
) -> Result<()>
```

Workers:
- Do not clone stages. If they are not Sync yet, we need to change them to be using interior Arc<Mutex or such
- Handle `needs_serial` stages by ensuring they have unique access
- Collect timing information
- Handle errors gracefully
- Support premature termination by signaling in done_tx

### Phase 2: Pipeline Integration

#### 2.1 Create Alternative Pipeline Entry Point
**File**: `mbf-fastq-processor/src/pipeline.rs`

Add new method to `RunStage2`:
```rust
impl RunStage2 {
    pub fn create_workpool_pipeline(self, parsed: &mut Config) -> RunStage3 {
        // Create workpool coordinator and workers instead of per-stage threads
    }
}
```

#### 2.2 Configuration Options
**File**: `mbf-fastq-processor/src/config.rs`

Add workpool-specific options:
```rust
pub struct WorkpoolOptions {
    pub worker_count: usize,
    pub max_blocks_in_flight: usize,
}

impl Default for WorkpoolOptions {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get(),
            max_blocks_in_flight: 100,
        }
    }
}
```

#### 2.3 CLI Integration
**File**: `mbf-fastq-processor/src/main.rs`

Add `--workpool` flag:
```rust
      .arg(
                    Arg::new("workpool")
                        .long("workpool")
                        .help("Use workpool instead of old pipeline")
                        .action(ArgAction::SetTrue),
```

### Phase 3: Workpool Coordinator Implementation

#### 3.1 Main Coordinator Loop
**File**: `mbf-fastq-processor/src/workpool.rs`

```rust
pub fn run_coordinator(
    mut coordinator: WorkpoolCoordinator,
    incoming_rx: Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>,
    todo_tx: Sender<WorkItem>,
    done_rx: Receiver<WorkResult>,
    output_tx: Sender<(usize, io::FastQBlocksCombined, Option<usize>)>,
    error_collector: Arc<Mutex<Vec<String>>>,
) {
    loop {
        select! {
            recv(incoming_rx) -> msg => {
                match msg {
                    Ok((block_no, block, expected_read_count)) => {
                        if block.is_final {
                            // Handle pipeline termination
                            coordinator.handle_final_block(block_no, output_tx);
                            break;
                        }
                        coordinator.process_incoming_block(block_no, block, expected_read_count);
                    }
                    Err(_) => break, // Input closed
                }
            }
            recv(done_rx) -> msg => {
                match msg {
                    Ok(work_result) => {
                        coordinator.process_completed_work(work_result);
                    }
                    Err(_) => break, // Workers closed
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
    }
}
```

#### 3.2 Serial Stage Ordering Logic
Key challenge: Ensure `needs_serial` stages process blocks in order

```rust
fn can_schedule_block_for_stage(&self, block_no: usize, stage_index: usize) -> bool {
    let stage_progress = &self.stage_progress[stage_index];
    
    if stage_progress.needs_serial {
        // For serial stages, can only process block N if block N-1 is complete
        block_no == stage_progress.highest_completed_block + 1
    } else {
        // Parallel stages can process any block
        true
    }
}
```

#### 3.3 Block Flow Management
```rust
fn advance_block_to_next_stage(&mut self, mut block_status: BlockStatus) {
    block_status.stage_index += 1;
    
    if block_status.stage_index >= self.stages.len() {
        // Block completed all stages - send to output
        self.send_to_output(block_status);
    } else {
        // Check if block can proceed to next stage
        if self.can_schedule_block_for_stage(block_status.block_no, block_status.stage_index) {
            self.schedule_work(block_status);
        } else {
            // Stall the block
            self.stalled_blocks.push(block_status);
        }
    }
}
```

### Phase 4: Worker Implementation

#### 4.1 Worker Thread Main Loop
```rust
fn worker_thread(
    worker_id: usize,
    todo_rx: Receiver<WorkItem>,
    done_tx: Sender<WorkResult>,
    input_info: transformations::InputInfo,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
) -> Result<()> {
    
    // Clone transformations for this worker
    let mut stages: Vec<Option<Transformation>> = vec![None; input_info.num_stages];
    
    while let Ok(work_item) = todo_rx.recv() {
        let result = process_work_item(
            work_item,
            &mut stages,
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
```

#### 4.2 Stage Execution Logic
```rust
fn process_work_item(
    work_item: WorkItem,
    stages: &mut Vec<Option<Transformation>>,
    input_info: &transformations::InputInfo,
    demultiplex_infos: &[(usize, OptDemultiplex)],
    timing_collector: &Arc<Mutex<Vec<crate::timing::StepTiming>>>,
) -> WorkResult {
    
    let stage_index = work_item.stage_index;
    
    // Get or clone the transformation
    let stage = if stages[stage_index].is_none() {
        // First time using this stage in this worker
        stages[stage_index] = Some(get_stage_clone(stage_index));
    }
    let stage = stages[stage_index].as_mut().unwrap();
    
    // Find appropriate demultiplex info
    let demultiplex_info = find_demultiplex_info(stage_index, demultiplex_infos);
    
    // Execute the transformation with timing
    let (wall_start, cpu_start) = crate::timing::StepTiming::start();
    
    let result = stage.apply(
        work_item.block,
        input_info,
        work_item.block_no,
        &demultiplex_info,
    );
    
    let timing = crate::timing::StepTiming::from_start(
        stage_index,
        stage.to_string(),
        wall_start,
        cpu_start,
    );
    
    timing_collector.lock().unwrap().push(timing);
    
    match result {
        Ok((result_block, stage_continue)) => {
            WorkResult {
                work_item,
                result_block,
                stage_continue,
                error: None,
            }
        }
        Err(e) => {
            WorkResult {
                work_item,
                result_block: io::FastQBlocksCombined::empty(),
                stage_continue: false,
                error: Some(e),
            }
        }
    }
}
```

### Phase 5: Integration and Testing

#### 5.1 Pipeline Mode Selection
**File**: `mbf-fastq-processor/src/pipeline.rs`

Modify `RunStage2::create_stage_threads` to delegate to workpool when enabled:

```rust
impl RunStage2 {
    pub fn create_stage_threads(self, parsed: &mut Config, use_workpool: bool) -> RunStage3 {
        if use_workpool {
            self.create_workpool_pipeline(parsed)
        } else {
            self.create_traditional_pipeline(parsed)
        }
    }
    
    // Rename existing method
    fn create_traditional_pipeline(self, parsed: &mut Config) -> RunStage3 {
        // existing implementation
    }
    
    fn create_workpool_pipeline(self, parsed: &mut Config) -> RunStage3 {
        // new workpool implementation
    }
}
```

#### 5.2 Configuration Validation
Add validation that workpool-specific options are only used with `--workpool` flag.

#### 5.3 Error Handling
Ensure error collection works properly:
- Workers report errors through `WorkResult`
- Coordinator aggregates errors
- Failed blocks still advance pipeline to avoid deadlocks
- Graceful shutdown on errors

#### 5.4 Testing Strategy

1. **Unit Tests**: Test coordinator logic in isolation
   - Block scheduling with serial/parallel stages
   - Ordering enforcement
   - Resource limiting

2. **Integration Tests**: Compare workpool vs traditional pipeline
  extend test_runner.rs run_test to run the verify step twice, once without --workpool and once with.

3. **Benchmark Comparison**: Performance analysis
   - will be done by the user

### Phase 6: Documentation and Optimization

#### 6.1 Configuration Documentation
Update TOML configuration guide with workpool options and performance tuning advice.

## Implementation Notes

### Critical Design Decisions

1. **Block Numbering**: Maintains existing `block_no` scheme for compatibility with output ordering

2. **Stage Cloning**: Workers clone transformations as needed, respecting `needs_serial` constraints

3. **Resource Management**: Global limits prevent memory exhaustion while maintaining throughput

4. **Error Propagation**: Errors don't stop the pipeline but are collected and reported

5. **Graceful Shutdown**: Proper cleanup when pipeline terminates early

### Compatibility Requirements

- Must produce identical output to traditional pipeline
- Support all existing transformation types
- Maintain demultiplexing behavior
- Preserve timing collection
- Support benchmark mode
- Handle all input modes (segmented, interleaved)

### Performance Goals

- Reduce thread oversubscription in high-stage pipelines
- Improve resource utilization through work stealing
- Maintain or improve throughput
- Better control over memory usage

This implementation provides a solid foundation for workpool-based pipeline processing while maintaining full compatibility with the existing system.
