// SPDX-License-Identifier: GPL-2.0

//! Kernel thread management

use alloc::{boxed::Box, string::String, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::error::Result;
use crate::sync::Spinlock;
use crate::{error, info};

/// Kernel thread ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KthreadId(u32);

/// Kernel thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KthreadState {
	Running,
	Sleeping,
	Stopped,
	Dead,
}

/// Kernel thread function type
pub type KthreadFn = fn();

/// Kernel thread descriptor
#[derive(Debug)]
pub struct Kthread {
	pub id: KthreadId,
	pub name: String,
	pub state: KthreadState,
	pub function: KthreadFn,
	// TODO: Add stack pointer, register context, etc.
}

impl Kthread {
	pub fn new(id: KthreadId, name: String, function: KthreadFn) -> Self {
		Self {
			id,
			name,
			state: KthreadState::Running,
			function,
		}
	}
}

/// Global kernel thread manager
static KTHREAD_MANAGER: Spinlock<KthreadManager> = Spinlock::new(KthreadManager::new());
static NEXT_KTHREAD_ID: AtomicU32 = AtomicU32::new(1);

/// Kernel thread manager
struct KthreadManager {
	threads: Vec<Kthread>,
}

impl KthreadManager {
	const fn new() -> Self {
		Self {
			threads: Vec::new(),
		}
	}

	fn spawn(&mut self, name: String, function: KthreadFn) -> KthreadId {
		let id = KthreadId(NEXT_KTHREAD_ID.fetch_add(1, Ordering::SeqCst));
		let thread = Kthread::new(id, name, function);

		self.threads.push(thread);
		id
	}

	fn get_thread(&self, id: KthreadId) -> Option<&Kthread> {
		self.threads.iter().find(|t| t.id == id)
	}

	fn get_thread_mut(&mut self, id: KthreadId) -> Option<&mut Kthread> {
		self.threads.iter_mut().find(|t| t.id == id)
	}
}

/// Spawn a new kernel thread
pub fn kthread_run(name: &str, function: KthreadFn) -> Result<KthreadId> {
	let mut manager = KTHREAD_MANAGER.lock();
	let id = manager.spawn(String::from(name), function);

	info!("Spawned kernel thread: {} (ID: {:?})", name, id);
	Ok(id)
}

/// Get current thread ID (simplified - always returns kernel thread 0 for now)
pub fn current_kthread_id() -> KthreadId {
	KthreadId(0)
}

/// Initialize kernel thread subsystem
pub fn init_kthreads() -> Result<()> {
	info!("Initializing kernel thread subsystem");

	// Spawn idle thread
	kthread_run("idle", idle_thread)?;

	// Spawn a test thread
	kthread_run("test", test_thread)?;

	info!("Kernel thread subsystem initialized");
	Ok(())
}

/// Idle kernel thread - runs when no other threads are active
fn idle_thread() {
	info!("Idle kernel thread started");

	loop {
		// In a real implementation, this would:
		// 1. Check for runnable threads
		// 2. Switch to them if available
		// 3. Otherwise, halt the CPU until interrupt

		#[cfg(target_arch = "x86_64")]
		unsafe {
			core::arch::asm!("hlt");
		}

		#[cfg(not(target_arch = "x86_64"))]
		core::hint::spin_loop();
	}
}

/// Test kernel thread
fn test_thread() {
	info!("Test kernel thread started");

	let mut counter = 0u32;
	loop {
		counter += 1;

		if counter % 10000000 == 0 {
			info!("Test thread tick: {}", counter);
		}

		// Simple delay
		for _ in 0..1000 {
			core::hint::spin_loop();
		}

		// Stop after a while to avoid spam
		if counter > 50000000 {
			info!("Test thread finished");
			break;
		}
	}
}

/// Simple cooperative yielding (simplified scheduler)
pub fn kthread_yield() {
	// In a real implementation, this would:
	// 1. Save current thread context
	// 2. Select next runnable thread
	// 3. Switch to it

	// For now, just do nothing - we don't have real threading yet
	core::hint::spin_loop();
}

/// Put current thread to sleep
pub fn kthread_sleep(duration_ms: u64) {
	// In a real implementation, this would:
	// 1. Set thread state to sleeping
	// 2. Set wake-up time
	// 3. Switch to another thread

	// For now, just busy wait (not efficient, but simple)
	let target_ticks = crate::time::get_jiffies() + (duration_ms * crate::time::HZ / 1000);

	while crate::time::get_jiffies().0 < target_ticks.0 {
		core::hint::spin_loop();
	}
}

/// Sleep for specified number of jiffies
pub fn sleep_for_jiffies(jiffies: u64) {
	let target_time = crate::time::get_jiffies() + jiffies;
	while crate::time::get_jiffies().0 < target_time.0 {
		core::hint::spin_loop();
	}
}

/// Get current thread count for diagnostics
pub fn get_thread_count() -> Result<usize> {
	let manager = KTHREAD_MANAGER.lock();
	Ok(manager.threads.len())
}
