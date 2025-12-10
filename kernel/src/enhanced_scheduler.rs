// SPDX-License-Identifier: GPL-2.0

//! Enhanced preemptive scheduler with improved context switching

use alloc::{
	collections::{BTreeMap, VecDeque},
	string::{String, ToString},
	vec::Vec,
};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::time::get_jiffies;
use crate::types::{Jiffies, Tid};

/// Preemption counter
static PREEMPTION_COUNT: AtomicU64 = AtomicU64::new(0);

/// Get preemption count
pub fn get_preemption_count() -> u64 {
	PREEMPTION_COUNT.load(Ordering::Relaxed)
}

/// Increment preemption counter
pub fn increment_preemption_count() {
	PREEMPTION_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Enhanced task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
	Running,
	Ready,
	Blocked,
	Sleeping,
	Zombie,
	Dead,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
	Critical = 0,
	High = 1,
	Normal = 2,
	Low = 3,
	Background = 4,
}

/// Enhanced task structure for better scheduling
#[derive(Debug, Clone)]
pub struct Task {
	pub tid: Tid,
	pub name: String,
	pub state: TaskState,
	pub priority: Priority,
	pub vruntime: u64,                // Virtual runtime for fair scheduling
	pub exec_time: u64,               // Total execution time
	pub sleep_until: Option<Jiffies>, // Wake up time if sleeping
	pub last_scheduled: Jiffies,      // Last time this task was scheduled
	pub cpu_affinity: u32,            // CPU affinity mask
	pub nice: i8,                     // Nice value (-20 to 19)
	pub preempt_count: u32,           // Preemption counter
}

impl Task {
	pub fn new(tid: Tid, name: String, priority: Priority) -> Self {
		let now = get_jiffies();
		Self {
			tid,
			name,
			state: TaskState::Ready,
			priority,
			vruntime: 0,
			exec_time: 0,
			sleep_until: None,
			last_scheduled: now,
			cpu_affinity: 0xFFFFFFFF, // All CPUs by default
			nice: 0,
			preempt_count: 0,
		}
	}

	/// Check if task is runnable
	pub fn is_runnable(&self) -> bool {
		match self.state {
			TaskState::Ready | TaskState::Running => true,
			TaskState::Sleeping => {
				if let Some(wake_time) = self.sleep_until {
					get_jiffies() >= wake_time
				} else {
					false
				}
			}
			_ => false,
		}
	}

	/// Update virtual runtime for fair scheduling
	pub fn update_vruntime(&mut self, delta: u64) {
		// Apply nice value weighting
		let weight = nice_to_weight(self.nice);
		self.vruntime += (delta * 1024) / weight;
		self.exec_time += delta;
	}

	/// Wake up sleeping task
	pub fn wake_up(&mut self) {
		if self.state == TaskState::Sleeping {
			self.state = TaskState::Ready;
			self.sleep_until = None;
		}
	}
}

/// Convert nice value to weight for scheduling calculations
fn nice_to_weight(nice: i8) -> u64 {
	// Weight table based on nice values (exponential scale)
	match nice {
		-20..=-15 => 88761,
		-14..=-10 => 71755,
		-9..=-5 => 56483,
		-4..=0 => 1024,
		1..=5 => 820,
		6..=10 => 655,
		11..=15 => 526,
		16..=19 => 423,
		_ => 1024, // Default weight
	}
}

/// Run queue for a specific priority level
#[derive(Debug)]
struct RunQueue {
	tasks: VecDeque<Tid>,
	total_weight: u64,
	min_vruntime: u64,
}

impl RunQueue {
	fn new() -> Self {
		Self {
			tasks: VecDeque::new(),
			total_weight: 0,
			min_vruntime: 0,
		}
	}

	fn add_task(&mut self, tid: Tid) {
		if !self.tasks.contains(&tid) {
			self.tasks.push_back(tid);
		}
	}

	fn remove_task(&mut self, tid: Tid) -> bool {
		if let Some(pos) = self.tasks.iter().position(|&t| t == tid) {
			self.tasks.remove(pos);
			true
		} else {
			false
		}
	}

	fn next_task(&mut self) -> Option<Tid> {
		self.tasks.pop_front()
	}

	fn is_empty(&self) -> bool {
		self.tasks.is_empty()
	}
}

/// Enhanced scheduler with preemptive multitasking
pub struct EnhancedScheduler {
	tasks: BTreeMap<Tid, Task>,
	run_queues: BTreeMap<Priority, RunQueue>,
	current_task: Option<Tid>,
	idle_task: Option<Tid>,
	next_tid: AtomicU64,
	total_context_switches: AtomicU64,
	preemption_enabled: AtomicBool,
	time_slice: u64, // Time slice in jiffies
}

impl EnhancedScheduler {
	pub fn new() -> Self {
		let mut run_queues = BTreeMap::new();
		run_queues.insert(Priority::Critical, RunQueue::new());
		run_queues.insert(Priority::High, RunQueue::new());
		run_queues.insert(Priority::Normal, RunQueue::new());
		run_queues.insert(Priority::Low, RunQueue::new());
		run_queues.insert(Priority::Background, RunQueue::new());

		Self {
			tasks: BTreeMap::new(),
			run_queues,
			current_task: None,
			idle_task: None,
			next_tid: AtomicU64::new(1),
			total_context_switches: AtomicU64::new(0),
			preemption_enabled: AtomicBool::new(true),
			time_slice: 10, // 10ms default time slice
		}
	}

	/// Add a new task to the scheduler
	pub fn add_task(&mut self, name: String, priority: Priority) -> Result<Tid> {
		let tid = Tid(self.next_tid.fetch_add(1, Ordering::SeqCst) as u32);
		let task = Task::new(tid, name, priority);

		self.tasks.insert(tid, task);

		if let Some(queue) = self.run_queues.get_mut(&priority) {
			queue.add_task(tid);
		}

		Ok(tid)
	}

	/// Remove a task from the scheduler
	pub fn remove_task(&mut self, tid: Tid) -> Result<()> {
		if let Some(task) = self.tasks.remove(&tid) {
			if let Some(queue) = self.run_queues.get_mut(&task.priority) {
				queue.remove_task(tid);
			}

			if self.current_task == Some(tid) {
				self.current_task = None;
			}

			Ok(())
		} else {
			Err(Error::NotFound)
		}
	}

	/// Get the next task to run
	pub fn schedule(&mut self) -> Option<Tid> {
		// Check if current task can continue running
		if let Some(current_tid) = self.current_task {
			if let Some(current_task) = self.tasks.get(&current_tid) {
				if current_task.is_runnable() && !self.should_preempt(current_task)
				{
					return Some(current_tid);
				}
			}
		}

		// Find the next task to run (priority-based with fair scheduling within
		// priority)
		for priority in [
			Priority::Critical,
			Priority::High,
			Priority::Normal,
			Priority::Low,
			Priority::Background,
		] {
			if let Some(queue) = self.run_queues.get_mut(&priority) {
				if !queue.is_empty() {
					// For fair scheduling, pick task with lowest vruntime
					let mut best_task = None;
					let mut min_vruntime = u64::MAX;

					for &tid in &queue.tasks {
						if let Some(task) = self.tasks.get(&tid) {
							if task.is_runnable()
								&& task.vruntime < min_vruntime
							{
								min_vruntime = task.vruntime;
								best_task = Some(tid);
							}
						}
					}

					if let Some(tid) = best_task {
						// Remove from current position and add to back for
						// round-robin
						queue.remove_task(tid);
						queue.add_task(tid);
						return Some(tid);
					}
				}
			}
		}

		// No runnable tasks, return idle task or None
		self.idle_task
	}

	/// Check if current task should be preempted
	fn should_preempt(&self, current_task: &Task) -> bool {
		if !self.preemption_enabled.load(Ordering::SeqCst) {
			return false;
		}

		let now = get_jiffies();
		let time_running = now - current_task.last_scheduled;

		// Preempt if time slice exceeded
		if time_running.as_u64() > self.time_slice {
			return true;
		}

		// Preempt if higher priority task is available
		for priority in [Priority::Critical, Priority::High] {
			if priority < current_task.priority {
				if let Some(queue) = self.run_queues.get(&priority) {
					if !queue.is_empty() {
						return true;
					}
				}
			}
		}

		false
	}

	/// Switch to a new task
	pub fn switch_to(&mut self, tid: Tid) -> Result<()> {
		if let Some(task) = self.tasks.get_mut(&tid) {
			task.state = TaskState::Running;
			task.last_scheduled = get_jiffies();
			self.current_task = Some(tid);
			self.total_context_switches.fetch_add(1, Ordering::SeqCst);
			Ok(())
		} else {
			Err(Error::NotFound)
		}
	}

	/// Put current task to sleep for specified duration
	pub fn sleep(&mut self, duration_jiffies: u64) -> Result<()> {
		if let Some(current_tid) = self.current_task {
			if let Some(task) = self.tasks.get_mut(&current_tid) {
				task.state = TaskState::Sleeping;
				task.sleep_until = Some(get_jiffies() + duration_jiffies);
				self.current_task = None;
				Ok(())
			} else {
				Err(Error::NotFound)
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	/// Wake up sleeping tasks
	pub fn wake_up_sleepers(&mut self) {
		let now = get_jiffies();

		for task in self.tasks.values_mut() {
			if task.state == TaskState::Sleeping {
				if let Some(wake_time) = task.sleep_until {
					if now >= wake_time {
						task.wake_up();

						// Add back to run queue
						if let Some(queue) =
							self.run_queues.get_mut(&task.priority)
						{
							queue.add_task(task.tid);
						}
					}
				}
			}
		}
	}

	/// Update virtual runtime for current task
	pub fn update_current_task(&mut self, time_delta: u64) {
		if let Some(current_tid) = self.current_task {
			if let Some(task) = self.tasks.get_mut(&current_tid) {
				task.update_vruntime(time_delta);
			}
		}
	}

	/// Get scheduler statistics
	pub fn get_stats(&self) -> SchedulerStats {
		let total_tasks = self.tasks.len();
		let runnable_tasks = self
			.tasks
			.values()
			.filter(|task| task.is_runnable())
			.count();
		let sleeping_tasks = self
			.tasks
			.values()
			.filter(|task| task.state == TaskState::Sleeping)
			.count();

		SchedulerStats {
			total_tasks,
			runnable_tasks,
			sleeping_tasks,
			context_switches: self.total_context_switches.load(Ordering::SeqCst),
			preemption_enabled: self.preemption_enabled.load(Ordering::SeqCst),
			current_task: self.current_task,
		}
	}

	/// Enable/disable preemption
	pub fn set_preemption(&self, enabled: bool) {
		self.preemption_enabled.store(enabled, Ordering::SeqCst);
	}

	/// Get current task
	pub fn current_task(&self) -> Option<Tid> {
		self.current_task
	}

	/// Get task by ID
	pub fn get_task(&self, tid: Tid) -> Option<&Task> {
		self.tasks.get(&tid)
	}

	/// Set task priority
	pub fn set_priority(&mut self, tid: Tid, priority: Priority) -> Result<()> {
		if let Some(task) = self.tasks.get_mut(&tid) {
			let old_priority = task.priority;
			task.priority = priority;

			// Move task between run queues
			if let Some(old_queue) = self.run_queues.get_mut(&old_priority) {
				old_queue.remove_task(tid);
			}

			if task.is_runnable() {
				if let Some(new_queue) = self.run_queues.get_mut(&priority) {
					new_queue.add_task(tid);
				}
			}

			Ok(())
		} else {
			Err(Error::NotFound)
		}
	}

	/// Create idle task
	pub fn create_idle_task(&mut self) -> Result<Tid> {
		let tid = self.add_task("idle".to_string(), Priority::Background)?;
		self.idle_task = Some(tid);
		Ok(tid)
	}
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
	pub total_tasks: usize,
	pub runnable_tasks: usize,
	pub sleeping_tasks: usize,
	pub context_switches: u64,
	pub preemption_enabled: bool,
	pub current_task: Option<Tid>,
}

/// Global enhanced scheduler
static ENHANCED_SCHEDULER: Spinlock<Option<EnhancedScheduler>> = Spinlock::new(None);

/// Helper to get scheduler reference safely
fn with_scheduler<T, F>(f: F) -> Option<T>
where
	F: FnOnce(&mut EnhancedScheduler) -> T,
{
	let mut scheduler_option = ENHANCED_SCHEDULER.lock();
	if let Some(ref mut scheduler) = *scheduler_option {
		Some(f(scheduler))
	} else {
		None
	}
}

/// Helper to get read-only scheduler reference safely
fn with_scheduler_read<T, F>(f: F) -> Option<T>
where
	F: FnOnce(&EnhancedScheduler) -> T,
{
	let scheduler_option = ENHANCED_SCHEDULER.lock();
	if let Some(ref scheduler) = *scheduler_option {
		Some(f(scheduler))
	} else {
		None
	}
}

/// Initialize enhanced scheduler
pub fn init_enhanced_scheduler() -> Result<()> {
	let mut scheduler_option = ENHANCED_SCHEDULER.lock();
	if scheduler_option.is_none() {
		let mut scheduler = EnhancedScheduler::new();
		scheduler.create_idle_task()?;
		*scheduler_option = Some(scheduler);
	}
	crate::info!("Enhanced scheduler initialized");
	Ok(())
}

/// Schedule next task
pub fn schedule_next() -> Option<Tid> {
	with_scheduler(|scheduler| {
		scheduler.wake_up_sleepers();
		scheduler.schedule()
	})
	.flatten()
}

/// Add new task to scheduler
pub fn add_task(name: String, priority: Priority) -> Result<Tid> {
	with_scheduler(|scheduler| scheduler.add_task(name, priority))
		.unwrap_or(Err(Error::NotInitialized))
}

/// Remove task from scheduler
pub fn remove_task(tid: Tid) -> Result<()> {
	with_scheduler(|scheduler| scheduler.remove_task(tid)).unwrap_or(Err(Error::NotInitialized))
}

/// Switch to specific task
pub fn switch_to_task(tid: Tid) -> Result<()> {
	with_scheduler(|scheduler| scheduler.switch_to(tid)).unwrap_or(Err(Error::NotInitialized))
}

/// Put current task to sleep
pub fn sleep_current_task(duration_jiffies: u64) -> Result<()> {
	with_scheduler(|scheduler| scheduler.sleep(duration_jiffies))
		.unwrap_or(Err(Error::NotInitialized))
}

/// Get scheduler statistics
pub fn get_scheduler_stats() -> SchedulerStats {
	with_scheduler_read(|scheduler| scheduler.get_stats()).unwrap_or(SchedulerStats {
		total_tasks: 0,
		runnable_tasks: 0,
		sleeping_tasks: 0,
		context_switches: 0,
		preemption_enabled: false,
		current_task: None,
	})
}

/// Update current task runtime
pub fn update_current_task_runtime(time_delta: u64) {
	with_scheduler(|scheduler| {
		scheduler.update_current_task(time_delta);
	});
}

/// Get current running task
pub fn get_current_task() -> Option<Tid> {
	with_scheduler_read(|scheduler| scheduler.current_task()).flatten()
}

/// Set task priority
pub fn set_task_priority(tid: Tid, priority: Priority) -> Result<()> {
	with_scheduler(|scheduler| scheduler.set_priority(tid, priority))
		.unwrap_or(Err(Error::NotInitialized))
}

/// Enable or disable preemption
pub fn set_preemption_enabled(enabled: bool) {
	with_scheduler(|scheduler| {
		scheduler.set_preemption(enabled);
	});
}
