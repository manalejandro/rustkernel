// SPDX-License-Identifier: GPL-2.0

//! Task scheduler compatible with Linux kernel CFS (Completely Fair Scheduler)

use alloc::{
	collections::{BTreeMap, VecDeque},
	vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::x86_64::context::{switch_context, Context};
use crate::error::{Error, Result};
use crate::process::{Thread, PROCESS_TABLE};
use crate::sync::Spinlock;
use crate::time;
use crate::types::Tid;

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
	pub vruntime: u64,         // Virtual runtime for CFS
	pub exec_start: u64,       // Last execution start time
	pub sum_exec_runtime: u64, // Total execution time
	pub prev_sum_exec_runtime: u64,
	pub load_weight: u32, // Load weight for this entity
	pub runnable_weight: u32,
	pub on_rq: bool, // On run queue?
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
		0 => 1024, // Default weight
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
				self.min_vruntime =
					core::cmp::max(self.min_vruntime, *next_vruntime);
			} else {
				self.min_vruntime = se.vruntime;
			}

			Some(se)
		} else {
			None
		}
	}

	/// Update minimum virtual runtime
	pub fn update_min_vruntime(&mut self) {
		if let Some((&next_vruntime, _)) = self.tasks_timeline.iter().next() {
			self.min_vruntime = core::cmp::max(self.min_vruntime, next_vruntime);
		}
	}
}

/// Real-time run queue  
#[derive(Debug)]
pub struct RtRunQueue {
	runqueue: VecDeque<SchedEntity>,
	nr_running: u32,
}

impl RtRunQueue {
	pub const fn new() -> Self {
		Self {
			runqueue: VecDeque::new(),
			nr_running: 0,
		}
	}

	pub fn enqueue_task(&mut self, se: SchedEntity) {
		self.runqueue.push_back(se);
		self.nr_running += 1;
	}

	pub fn dequeue_task(&mut self, se: &SchedEntity) -> bool {
		if let Some(pos) = self.runqueue.iter().position(|task| task.tid == se.tid) {
			self.nr_running -= 1;
			self.runqueue.remove(pos);
			true
		} else {
			false
		}
	}

	pub fn pick_next_task(&mut self) -> Option<SchedEntity> {
		if self.nr_running > 0 {
			self.nr_running -= 1;
			self.runqueue.pop_front()
		} else {
			None
		}
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
			SchedulerPolicy::Normal
			| SchedulerPolicy::Batch
			| SchedulerPolicy::Idle => {
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
			SchedulerPolicy::Normal
			| SchedulerPolicy::Batch
			| SchedulerPolicy::Idle => self.cfs.dequeue_task(se),
			SchedulerPolicy::Fifo | SchedulerPolicy::RoundRobin => {
				self.rt.dequeue_task(se)
			}
			SchedulerPolicy::Deadline => self.cfs.dequeue_task(se),
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
	cfs: CfsRunQueue,
	rt: RtRunQueue,
	current: Option<Tid>,
	nr_switches: u64,
}

impl Scheduler {
	const fn new() -> Self {
		Self {
			run_queues: Vec::new(),
			nr_cpus: 1, // Single CPU for now
			entities: BTreeMap::new(),
			need_resched: false,
			cfs: CfsRunQueue {
				tasks_timeline: BTreeMap::new(),
				min_vruntime: 0,
				nr_running: 0,
				load_weight: 0,
				runnable_weight: 0,
			},
			rt: RtRunQueue {
				runqueue: VecDeque::new(),
				nr_running: 0,
			},
			current: None,
			nr_switches: 0,
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
				MAX_NICE,
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

	/// Pick next task to run
	fn pick_next_task(&mut self) -> Option<Tid> {
		// Try CFS first
		if let Some(se) = self.cfs.pick_next_task() {
			self.current = Some(se.tid);
			return Some(se.tid);
		}

		// Then try RT
		if let Some(se) = self.rt.pick_next_task() {
			self.current = Some(se.tid);
			return Some(se.tid);
		}

		None
	}

	/// Switch to a task
	fn switch_to(&mut self, tid: Tid) {
		// Save current task's context
		if let Some(current_tid) = self.current {
			if current_tid != tid {
				// Look up current and next threads
				// We need to use a scope to ensure the lock is dropped before switching
				let (current_ctx_ptr, next_ctx_ptr) = {
					let mut process_table = PROCESS_TABLE.lock();

					let (current_thread, next_thread) = process_table
						.find_two_threads_mut(current_tid, tid);

					let current_ptr = if let Some(t) = current_thread {
						&mut t.context as *mut Context
					} else {
						return; // Current thread not found?
					};

					let next_ptr = if let Some(t) = next_thread {
						&t.context as *const Context
					} else {
						return; // Next thread not found
					};

					(current_ptr, next_ptr)
				};

				// Update scheduler state
				self.current = Some(tid);
				self.nr_switches += 1;

				// Perform the context switch
				// SAFETY: We have valid pointers to the contexts and we've dropped the lock
				unsafe {
					switch_context(&mut *current_ctx_ptr, &*next_ctx_ptr);
				}

				return;
			}
		}

		// First task or same task
		self.current = Some(tid);
		self.nr_switches += 1;
	}

	/// Set need resched flag
	fn set_need_resched(&mut self) {
		self.need_resched = true;
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
pub fn add_task(pid: crate::types::Pid) -> Result<()> {
	let mut scheduler = SCHEDULER.lock();

	// Create a scheduler entity for the process
	let tid = crate::types::Tid(pid.0); // Simple mapping for now
	let se = SchedEntity::new(tid, SchedulerPolicy::Normal, DEFAULT_PRIO);

	// Add to CFS runqueue
	scheduler.cfs.enqueue_task(se);

	Ok(())
}

/// Remove a task from the scheduler
pub fn remove_task(pid: crate::types::Pid) -> Result<()> {
	let mut scheduler = SCHEDULER.lock();

	// Remove from all runqueues
	let tid = crate::types::Tid(pid.0);

	// Create a minimal SchedEntity for removal
	let se = SchedEntity::new(tid, SchedulerPolicy::Normal, DEFAULT_PRIO);
	scheduler.cfs.dequeue_task(&se);
	scheduler.rt.dequeue_task(&se);

	Ok(())
}

/// Schedule next task (called from syscall exit or timer interrupt)
pub fn schedule() {
	let mut scheduler = SCHEDULER.lock();

	// Pick next task to run
	if let Some(next) = scheduler.pick_next_task() {
		// Switch to next task
		scheduler.switch_to(next);
	}
}

/// Get current running task
pub fn current_task() -> Option<crate::types::Pid> {
	let scheduler = SCHEDULER.lock();
	scheduler.current.map(|tid| crate::types::Pid(tid.0))
}

/// Yield current task (alias for yield_task)
pub fn yield_now() {
	yield_task();
}

/// Yield current task
pub fn yield_task() {
	let mut scheduler = SCHEDULER.lock();
	scheduler.set_need_resched();
}

/// Sleep current task for specified duration
pub fn sleep_task(duration_ms: u64) {
	// TODO: implement proper sleep mechanism with timer integration
	// For now, just yield
	yield_task();
}

/// Wake up a task
pub fn wake_task(pid: crate::types::Pid) -> Result<()> {
	let mut scheduler = SCHEDULER.lock();
	let tid = crate::types::Tid(pid.0);

	// TODO: Move from wait queue to runqueue
	// For now, just ensure it's in the runqueue
	let se = SchedEntity::new(tid, SchedulerPolicy::Normal, DEFAULT_PRIO);
	scheduler.cfs.enqueue_task(se);

	Ok(())
}

/// Set task priority
pub fn set_task_priority(pid: crate::types::Pid, priority: i32) -> Result<()> {
	let mut scheduler = SCHEDULER.lock();
	let tid = crate::types::Tid(pid.0);

	// TODO: Update priority in runqueue
	// This would require finding the task and updating its priority

	Ok(())
}

/// Get scheduler statistics
pub fn get_scheduler_stats() -> SchedulerStats {
	let scheduler = SCHEDULER.lock();
	SchedulerStats {
		total_tasks: (scheduler.cfs.nr_running + scheduler.rt.nr_running) as usize,
		running_tasks: if scheduler.current.is_some() { 1 } else { 0 },
		context_switches: scheduler.nr_switches,
		load_average: scheduler.cfs.load_weight as f64 / 1024.0,
	}
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
	pub total_tasks: usize,
	pub running_tasks: usize,
	pub context_switches: u64,
	pub load_average: f64,
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
				if current.sum_exec_runtime - current.prev_sum_exec_runtime
					>= time_slice
				{
					scheduler.set_need_resched();
				}
			}
		}
	}
}

/// Perform a manual context switch to a specific task
/// This is used by the enhanced scheduler to execute its scheduling decisions
pub fn context_switch_to(tid: Tid) {
	let mut scheduler = SCHEDULER.lock();
	scheduler.switch_to(tid);
}
