// SPDX-License-Identifier: GPL-2.0

//! Task scheduler compatible with Linux kernel CFS (Completely Fair Scheduler)

use crate::error::{Error, Result};
use crate::process::{Process, Thread, ProcessState, ThreadContext};
use crate::types::{Pid, Tid, Nanoseconds};
use crate::sync::Spinlock;
use crate::time;
use alloc::{collections::{BTreeMap, VecDeque}, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

/// Scheduler policies - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerPolicy {
    Normal = 0,     // SCHED_NORMAL
    Fifo = 1,       // SCHED_FIFO  
    RoundRobin = 2, // SCHED_RR
    Batch = 3,      // SCHED_BATCH
    Idle = 5,       // SCHED_IDLE
    Deadline = 6,   // SCHED_DEADLINE
}

/// Scheduler priority levels
pub const MAX_PRIO: i32 = 140;
pub const MAX_USER_RT_PRIO: i32 = 100;
pub const MAX_RT_PRIO: i32 = MAX_USER_RT_PRIO;
pub const DEFAULT_PRIO: i32 = MAX_RT_PRIO + 20;
pub const MIN_NICE: i32 = -20;
pub const MAX_NICE: i32 = 19;

/// Convert nice value to priority
pub fn nice_to_prio(nice: i32) -> i32 {
    DEFAULT_PRIO + nice
}

/// Convert priority to nice value
pub fn prio_to_nice(prio: i32) -> i32 {
    prio - DEFAULT_PRIO
}

/// Scheduler entity - represents a schedulable unit
#[derive(Debug, Clone)]
pub struct SchedEntity {
    pub tid: Tid,
    pub policy: SchedulerPolicy,
    pub priority: i32,
    pub nice: i32,
    pub vruntime: u64,      // Virtual runtime for CFS
    pub exec_start: u64,    // Last execution start time
    pub sum_exec_runtime: u64, // Total execution time
    pub prev_sum_exec_runtime: u64,
    pub load_weight: u32,   // Load weight for this entity
    pub runnable_weight: u32,
    pub on_rq: bool,        // On run queue?
}

impl SchedEntity {
    pub fn new(tid: Tid, policy: SchedulerPolicy, nice: i32) -> Self {
        let priority = nice_to_prio(nice);
        Self {
            tid,
            policy,
            priority,
            nice,
            vruntime: 0,
            exec_start: 0,
            sum_exec_runtime: 0,
            prev_sum_exec_runtime: 0,
            load_weight: nice_to_weight(nice),
            runnable_weight: nice_to_weight(nice),
            on_rq: false,
        }
    }
    
    /// Update virtual runtime
    pub fn update_vruntime(&mut self, delta: u64) {
        // Virtual runtime is weighted by load
        let weighted_delta = delta * 1024 / self.load_weight as u64;
        self.vruntime += weighted_delta;
    }
}

/// Convert nice value to load weight (Linux compatible)
fn nice_to_weight(nice: i32) -> u32 {
    // Linux nice-to-weight table (simplified)
    match nice {
        -20 => 88761,
        -19 => 71755,
        -18 => 56483,
        -17 => 46273,
        -16 => 36291,
        -15 => 29154,
        -14 => 23254,
        -13 => 18705,
        -12 => 14949,
        -11 => 11916,
        -10 => 9548,
        -9 => 7620,
        -8 => 6100,
        -7 => 4904,
        -6 => 3906,
        -5 => 3121,
        -4 => 2501,
        -3 => 1991,
        -2 => 1586,
        -1 => 1277,
        0 => 1024,  // Default weight
        1 => 820,
        2 => 655,
        3 => 526,
        4 => 423,
        5 => 335,
        6 => 272,
        7 => 215,
        8 => 172,
        9 => 137,
        10 => 110,
        11 => 87,
        12 => 70,
        13 => 56,
        14 => 45,
        15 => 36,
        16 => 29,
        17 => 23,
        18 => 18,
        19 => 15,
        _ => 1024, // Default for out-of-range values
    }
}

/// CFS (Completely Fair Scheduler) run queue
#[derive(Debug)]
pub struct CfsRunQueue {
    tasks_timeline: BTreeMap<u64, SchedEntity>, // Red-black tree equivalent
    min_vruntime: u64,
    nr_running: u32,
    load_weight: u64,
    runnable_weight: u64,
}

impl CfsRunQueue {
    pub fn new() -> Self {
        Self {
            tasks_timeline: BTreeMap::new(),
            min_vruntime: 0,
            nr_running: 0,
            load_weight: 0,
            runnable_weight: 0,
        }
    }
    
    /// Add task to run queue
    pub fn enqueue_task(&mut self, mut se: SchedEntity) {
        // Place entity in timeline
        if !se.on_rq {
            se.on_rq = true;
            
            // Ensure vruntime is not too far behind
            if se.vruntime < self.min_vruntime {
                se.vruntime = self.min_vruntime;
            }
            
            self.tasks_timeline.insert(se.vruntime, se.clone());
            self.nr_running += 1;
            self.load_weight += se.load_weight as u64;
            self.runnable_weight += se.runnable_weight as u64;
        }
    }
    
    /// Remove task from run queue
    pub fn dequeue_task(&mut self, se: &SchedEntity) -> bool {
        if se.on_rq {
            self.tasks_timeline.remove(&se.vruntime);
            self.nr_running -= 1;
            self.load_weight -= se.load_weight as u64;
            self.runnable_weight -= se.runnable_weight as u64;
            true
        } else {
            false
        }
    }
    
    /// Pick next task to run
    pub fn pick_next_task(&mut self) -> Option<SchedEntity> {
        // Pick leftmost task (smallest vruntime)
        if let Some((vruntime, se)) = self.tasks_timeline.iter().next() {
            let se = se.clone();
            let vruntime = *vruntime;
            self.tasks_timeline.remove(&vruntime);
            self.nr_running -= 1;
            self.load_weight -= se.load_weight as u64;
            self.runnable_weight -= se.runnable_weight as u64;
            
            // Update min_vruntime
            if let Some((next_vruntime, _)) = self.tasks_timeline.iter().next() {
                self.min_vruntime = core::cmp::max(self.min_vruntime, *next_vruntime);
            } else {
                self.min_vruntime = se.vruntime;
            }
            
            Some(se)
        } else {
            None
        }
    }
    
    /// Check if run queue is empty
    pub fn is_empty(&self) -> bool {
        self.nr_running == 0
    }
}

/// Real-time run queue (for FIFO/RR scheduling)
#[derive(Debug)]
pub struct RtRunQueue {
    active: [VecDeque<SchedEntity>; MAX_RT_PRIO as usize],
    rt_nr_running: u32,
    highest_prio: i32,
}

impl RtRunQueue {
    pub fn new() -> Self {
        const EMPTY_QUEUE: VecDeque<SchedEntity> = VecDeque::new();
        Self {
            active: [EMPTY_QUEUE; MAX_RT_PRIO as usize],
            rt_nr_running: 0,
            highest_prio: MAX_RT_PRIO,
        }
    }
    
    pub fn enqueue_task(&mut self, se: SchedEntity) {
        let prio = se.priority as usize;
        if prio < MAX_RT_PRIO as usize {
            self.active[prio].push_back(se);
            self.rt_nr_running += 1;
            if (prio as i32) < self.highest_prio {
                self.highest_prio = prio as i32;
            }
        }
    }
    
    pub fn dequeue_task(&mut self, se: &SchedEntity) -> bool {
        let prio = se.priority as usize;
        if prio < MAX_RT_PRIO as usize {
            if let Some(pos) = self.active[prio].iter().position(|x| x.tid == se.tid) {
                self.active[prio].remove(pos);
                self.rt_nr_running -= 1;
                
                // Update highest_prio if this queue is now empty
                if self.active[prio].is_empty() && prio as i32 == self.highest_prio {
                    self.update_highest_prio();
                }
                return true;
            }
        }
        false
    }
    
    pub fn pick_next_task(&mut self) -> Option<SchedEntity> {
        if self.rt_nr_running > 0 {
            for prio in self.highest_prio as usize..MAX_RT_PRIO as usize {
                if let Some(se) = self.active[prio].pop_front() {
                    self.rt_nr_running -= 1;
                    
                    // For round-robin, re-enqueue at the end
                    if se.policy == SchedulerPolicy::RoundRobin {
                        self.active[prio].push_back(se.clone());
                        self.rt_nr_running += 1;
                    }
                    
                    if self.active[prio].is_empty() && prio as i32 == self.highest_prio {
                        self.update_highest_prio();
                    }
                    
                    return Some(se);
                }
            }
        }
        None
    }
    
    fn update_highest_prio(&mut self) {
        self.highest_prio = MAX_RT_PRIO;
        for prio in 0..MAX_RT_PRIO as usize {
            if !self.active[prio].is_empty() {
                self.highest_prio = prio as i32;
                break;
            }
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.rt_nr_running == 0
    }
}

/// Per-CPU run queue
#[derive(Debug)]
pub struct RunQueue {
    pub cpu: u32,
    pub nr_running: u32,
    pub current: Option<SchedEntity>,
    pub cfs: CfsRunQueue,
    pub rt: RtRunQueue,
    pub idle_task: Option<SchedEntity>,
    pub clock: u64,
    pub clock_task: u64,
}

impl RunQueue {
    pub fn new(cpu: u32) -> Self {
        Self {
            cpu,
            nr_running: 0,
            current: None,
            cfs: CfsRunQueue::new(),
            rt: RtRunQueue::new(),
            idle_task: None,
            clock: 0,
            clock_task: 0,
        }
    }
    
    /// Update run queue clock
    pub fn update_rq_clock(&mut self) {
        self.clock = time::get_time_ns();
        self.clock_task = self.clock;
    }
    
    /// Enqueue a task
    pub fn enqueue_task(&mut self, se: SchedEntity) {
        match se.policy {
            SchedulerPolicy::Normal | SchedulerPolicy::Batch | SchedulerPolicy::Idle => {
                self.cfs.enqueue_task(se);
            }
            SchedulerPolicy::Fifo | SchedulerPolicy::RoundRobin => {
                self.rt.enqueue_task(se);
            }
            SchedulerPolicy::Deadline => {
                // TODO: implement deadline scheduler
                self.cfs.enqueue_task(se);
            }
        }
        self.nr_running += 1;
    }
    
    /// Dequeue a task
    pub fn dequeue_task(&mut self, se: &SchedEntity) -> bool {
        let result = match se.policy {
            SchedulerPolicy::Normal | SchedulerPolicy::Batch | SchedulerPolicy::Idle => {
                self.cfs.dequeue_task(se)
            }
            SchedulerPolicy::Fifo | SchedulerPolicy::RoundRobin => {
                self.rt.dequeue_task(se)
            }
            SchedulerPolicy::Deadline => {
                self.cfs.dequeue_task(se)
            }
        };
        
        if result {
            self.nr_running -= 1;
        }
        result
    }
    
    /// Pick next task to run
    pub fn pick_next_task(&mut self) -> Option<SchedEntity> {
        // Real-time tasks have priority
        if let Some(se) = self.rt.pick_next_task() {
            return Some(se);
        }
        
        // Then CFS tasks
        if let Some(se) = self.cfs.pick_next_task() {
            return Some(se);
        }
        
        // Finally, idle task
        self.idle_task.clone()
    }
}

/// Global scheduler state
static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());
static SCHEDULE_CLOCK: AtomicU64 = AtomicU64::new(0);

/// Main scheduler structure
struct Scheduler {
    run_queues: Vec<RunQueue>,
    nr_cpus: u32,
    entities: BTreeMap<Tid, SchedEntity>,
    need_resched: bool,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            run_queues: Vec::new(),
            nr_cpus: 1, // Single CPU for now
            entities: BTreeMap::new(),
            need_resched: false,
        }
    }
    
    fn init(&mut self) -> Result<()> {
        // Create run queues for each CPU
        for cpu in 0..self.nr_cpus {
            let mut rq = RunQueue::new(cpu);
            
            // Create idle task for this CPU
            let idle_se = SchedEntity::new(
                crate::process::allocate_tid(),
                SchedulerPolicy::Idle,
                MAX_NICE
            );
            rq.idle_task = Some(idle_se);
            
            self.run_queues.push(rq);
        }
        Ok(())
    }
    
    fn add_task(&mut self, tid: Tid, policy: SchedulerPolicy, nice: i32) {
        let se = SchedEntity::new(tid, policy, nice);
        self.entities.insert(tid, se.clone());
        
        // Add to CPU 0's run queue for simplicity
        if let Some(rq) = self.run_queues.get_mut(0) {
            rq.enqueue_task(se);
        }
    }
    
    fn remove_task(&mut self, tid: Tid) {
        if let Some(se) = self.entities.remove(&tid) {
            // Remove from all run queues
            for rq in &mut self.run_queues {
                rq.dequeue_task(&se);
            }
        }
    }
    
    fn schedule(&mut self) -> Option<Tid> {
        // Simple single-CPU scheduling for now
        if let Some(rq) = self.run_queues.get_mut(0) {
            rq.update_rq_clock();
            
            if let Some(se) = rq.pick_next_task() {
                rq.current = Some(se.clone());
                return Some(se.tid);
            }
        }
        None
    }
    
    fn set_need_resched(&mut self) {
        self.need_resched = true;
    }
    
    fn clear_need_resched(&mut self) {
        self.need_resched = false;
    }
}

/// Initialize the scheduler
pub fn init() -> Result<()> {
    let mut scheduler = SCHEDULER.lock();
    scheduler.init()?;
    
    crate::info!("Scheduler initialized with {} CPUs", scheduler.nr_cpus);
    Ok(())
}

/// Add a task to the scheduler
pub fn add_task(tid: Tid, policy: SchedulerPolicy, nice: i32) {
    let mut scheduler = SCHEDULER.lock();
    scheduler.add_task(tid, policy, nice);
}

/// Remove a task from the scheduler
pub fn remove_task(tid: Tid) {
    let mut scheduler = SCHEDULER.lock();
    scheduler.remove_task(tid);
}

/// Schedule the next task
pub fn schedule() -> Option<Tid> {
    let mut scheduler = SCHEDULER.lock();
    let result = scheduler.schedule();
    scheduler.clear_need_resched();
    result
}

/// Yield the current thread
pub fn yield_now() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.set_need_resched();
    // In a real implementation, this would trigger a context switch
}

/// Sleep for a given number of milliseconds
pub fn sleep_ms(ms: u64) {
    // TODO: implement proper sleep mechanism with timer integration
    // For now, just yield
    yield_now();
}

/// Set scheduler policy for a task
pub fn set_scheduler_policy(tid: Tid, policy: SchedulerPolicy, nice: i32) -> Result<()> {
    let mut scheduler = SCHEDULER.lock();
    
    if let Some(se) = scheduler.entities.get_mut(&tid) {
        se.policy = policy;
        se.nice = nice;
        se.priority = nice_to_prio(nice);
        se.load_weight = nice_to_weight(nice);
        se.runnable_weight = nice_to_weight(nice);
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}

/// Get scheduler statistics
pub fn get_scheduler_stats() -> (u32, u32, bool) {
    let scheduler = SCHEDULER.lock();
    let total_tasks = scheduler.entities.len() as u32;
    let running_tasks = scheduler.run_queues.iter().map(|rq| rq.nr_running).sum();
    (total_tasks, running_tasks, scheduler.need_resched)
}

/// Timer tick - called from timer interrupt
pub fn scheduler_tick() {
    SCHEDULE_CLOCK.fetch_add(1, Ordering::Relaxed);
    
    let mut scheduler = SCHEDULER.lock();
    
    // Update current task's runtime
    if let Some(rq) = scheduler.run_queues.get_mut(0) {
        if let Some(ref mut current) = rq.current {
            let now = rq.clock;
            let delta = now - current.exec_start;
            current.sum_exec_runtime += delta;
            current.update_vruntime(delta);
            current.exec_start = now;
            
            // Check if we need to reschedule
            // For CFS, check if current task has run long enough
            if current.policy == SchedulerPolicy::Normal {
                let time_slice = calculate_time_slice(current);
                if current.sum_exec_runtime - current.prev_sum_exec_runtime >= time_slice {
                    scheduler.set_need_resched();
                }
            }
        }
    }
}

/// Calculate time slice for a task based on its weight
fn calculate_time_slice(se: &SchedEntity) -> u64 {
    // Linux-like time slice calculation
    let sched_latency = 6_000_000; // 6ms in nanoseconds
    let min_granularity = 750_000; // 0.75ms in nanoseconds
    
    // Time slice proportional to weight
    let time_slice = sched_latency * se.load_weight as u64 / 1024;
    core::cmp::max(time_slice, min_granularity)
}
