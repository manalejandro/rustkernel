// SPDX-License-Identifier: GPL-2.0

//! Working kernel task implementation with actual functionality

use alloc::{
	boxed::Box,
	format,
	string::{String, ToString},
	vec::Vec,
};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::arch::x86_64::context::Context;
use crate::error::{Error, Result};
use crate::memory::kmalloc;
use crate::sync::Spinlock;
use crate::types::{Pid, Tid};

/// Task function type
pub type TaskFunction = fn() -> ();

/// Task state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
	Running,
	Ready,
	Blocked,
	Terminated,
}

/// Working task structure
#[derive(Debug, Clone)]
pub struct Task {
	pub tid: Tid,
	pub pid: Pid,
	pub name: String,
	pub state: TaskState,
	pub context: Context,
	pub stack_base: usize,
	pub stack_size: usize,
	pub priority: u8,
	pub cpu_time: u64,
	pub creation_time: u64,
}

impl Task {
	/// Create a new kernel task
	pub fn new_kernel_task(
		name: String,
		function: TaskFunction,
		stack_size: usize,
	) -> Result<Self> {
		// Use process subsystem to allocate TID to ensure uniqueness
		let tid = crate::process::allocate_tid();

		// Allocate stack
		let stack_ptr = kmalloc::kmalloc(stack_size)?;
		let stack_base = stack_ptr as usize;
		let stack_top = stack_base + stack_size;

		// Set up initial context
		let mut context = Context::new();
		context.rsp = (stack_top - 8) as u64; // Leave space for return address
		context.rip = function as usize as u64;
		context.rflags = 0x202; // Enable interrupts
		context.cs = 0x08; // Kernel code segment
		context.ds = 0x10; // Kernel data segment
		context.es = 0x10;
		context.fs = 0x10;
		context.gs = 0x10;
		context.ss = 0x10;

		// Write a dummy return address to stack (for if function returns)
		unsafe {
			let return_addr_ptr = (stack_top - 8) as *mut u64;
			*return_addr_ptr = task_exit_wrapper as usize as u64;
		}

		Ok(Task {
			tid,
			pid: Pid(0), // Kernel process
			name,
			state: TaskState::Ready,
			context,
			stack_base,
			stack_size,
			priority: 128, // Default priority
			cpu_time: 0,
			creation_time: crate::time::get_jiffies().0,
		})
	}

	/// Set task priority
	pub fn set_priority(&mut self, priority: u8) {
		self.priority = priority;
	}

	/// Update CPU time
	pub fn add_cpu_time(&mut self, time: u64) {
		self.cpu_time += time;
	}

	/// Check if task should be scheduled
	pub fn is_schedulable(&self) -> bool {
		matches!(self.state, TaskState::Ready | TaskState::Running)
	}

	/// Terminate task
	pub fn terminate(&mut self) {
		self.state = TaskState::Terminated;

		// Free stack memory
		unsafe {
			kmalloc::kfree(self.stack_base as *mut u8);
		}
	}
}

/// Wrapper function called when a task function returns
extern "C" fn task_exit_wrapper() -> ! {
	crate::info!("Kernel task exited normally");

	// Mark current task as terminated
	if let Some(current_tid) = crate::enhanced_scheduler::get_current_task() {
		let _ = crate::enhanced_scheduler::remove_task(current_tid);
	}

	// Yield to scheduler
	loop {
		crate::enhanced_scheduler::schedule_next();
		// If we get here, no other tasks to schedule
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}

/// Working task manager
pub struct TaskManager {
	tasks: Spinlock<Vec<Task>>,
	next_tid: AtomicU32,
}

impl TaskManager {
	pub const fn new() -> Self {
		Self {
			tasks: Spinlock::new(Vec::new()),
			next_tid: AtomicU32::new(1),
		}
	}

	/// Spawn a new kernel task
	pub fn spawn_kernel_task(
		&self,
		name: String,
		function: TaskFunction,
		stack_size: usize,
	) -> Result<Tid> {
		let task = Task::new_kernel_task(name.clone(), function, stack_size)?;
		let tid = task.tid;

		// Add to local task list
		self.tasks.lock().push(task.clone());

		// Add to PROCESS_TABLE so the low-level scheduler can find it
		crate::process::add_kernel_thread(
			tid,
			task.context,
			crate::memory::VirtAddr::new(task.context.rsp as usize),
		)?;

		// Add to enhanced scheduler
		crate::enhanced_scheduler::add_task(
			format!("task-{}", tid.0),
			crate::enhanced_scheduler::Priority::Normal,
		)?;

		crate::info!(
			"Spawned kernel task {} with TID {:?}",
			format!("task-{}", tid.0),
			tid
		);

		Ok(tid)
	}

	/// Get task by TID
	pub fn get_task(&self, tid: Tid) -> Option<Task> {
		self.tasks
			.lock()
			.iter()
			.find(|task| task.tid == tid)
			.cloned()
	}

	/// Update task state
	pub fn set_task_state(&self, tid: Tid, state: TaskState) -> Result<()> {
		let mut tasks = self.tasks.lock();
		match tasks.iter_mut().find(|task| task.tid == tid) {
			Some(task) => {
				task.state = state;
				Ok(())
			}
			None => Err(Error::NotFound),
		}
	}

	/// Get all tasks
	pub fn get_all_tasks(&self) -> Vec<Task> {
		self.tasks.lock().clone()
	}

	/// Clean up terminated tasks
	pub fn cleanup_terminated_tasks(&self) {
		let mut tasks = self.tasks.lock();
		tasks.retain(|task| task.state != TaskState::Terminated);
	}

	/// Get total number of tasks
	pub fn get_task_count(&self) -> usize {
		self.tasks.lock().len()
	}
}

/// Global task manager
static TASK_MANAGER: TaskManager = TaskManager::new();

/// Initialize task management
pub fn init_task_management() -> Result<()> {
	crate::info!("Task management initialized");
	Ok(())
}

/// Spawn a kernel task
pub fn spawn_kernel_task(name: String, function: TaskFunction, stack_size: usize) -> Result<Tid> {
	TASK_MANAGER.spawn_kernel_task(name, function, stack_size)
}

/// Create and spawn a kernel task (alias for compatibility)
pub fn create_kernel_task(name: &str, function: TaskFunction) -> Result<Tid> {
	spawn_kernel_task(name.to_string(), function, 8192) // 8KB default stack
}

/// Get task information
pub fn get_task_info(tid: Tid) -> Option<Task> {
	TASK_MANAGER.get_task(tid)
}

/// Set task state  
pub fn set_task_state(tid: Tid, state: TaskState) -> Result<()> {
	TASK_MANAGER.set_task_state(tid, state)
}

/// Get all tasks
pub fn get_all_tasks() -> Vec<Task> {
	TASK_MANAGER.get_all_tasks()
}

/// Clean up terminated tasks
pub fn cleanup_tasks() {
	TASK_MANAGER.cleanup_terminated_tasks();
}

/// Get total task count
pub fn get_task_count() -> usize {
	TASK_MANAGER.get_task_count()
}

/// Example kernel tasks for testing

/// Idle task that runs when no other tasks are ready
pub fn idle_task() {
	loop {
		unsafe {
			core::arch::asm!("hlt"); // Halt until interrupt
		}
	}
}

/// Heartbeat task that prints periodic messages
pub fn heartbeat_task() {
	let mut counter = 0;
	loop {
		counter += 1;
		crate::info!("Heartbeat: {}", counter);

		// Sleep for a while (simplified - just loop)
		for _ in 0..1_000_000 {
			unsafe {
				core::arch::asm!("pause");
			}
		}

		if counter >= 10 {
			crate::info!("Heartbeat task exiting after 10 beats");
			break;
		}
	}
}

/// Memory monitor task
pub fn memory_monitor_task() {
	loop {
		let stats = crate::memory::advanced_allocator::get_memory_stats();
		crate::info!(
			"Memory: allocated={} KB, peak={} KB, active_allocs={}",
			stats.current_allocated / 1024,
			stats.peak_usage / 1024,
			stats.active_allocations
		);

		// Sleep for a while
		for _ in 0..5_000_000 {
			unsafe {
				core::arch::asm!("pause");
			}
		}
	}
}

/// Performance monitor task
pub fn performance_monitor_task() {
	loop {
		let summary = crate::advanced_perf::get_performance_summary();
		let mut total_counters = 0;
		for (_, value) in &summary.counters {
			total_counters += value;
		}

		if total_counters > 0 {
			crate::info!(
				"Performance: {} events, {} profilers active",
				summary.total_events,
				summary.profilers.len()
			);
		}

		// Sleep for a while
		for _ in 0..10_000_000 {
			unsafe {
				core::arch::asm!("pause");
			}
		}
	}
}
