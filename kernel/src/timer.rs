// SPDX-License-Identifier: GPL-2.0

//! Timer interrupt handler for preemptive scheduling

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::enhanced_scheduler;
use crate::sync::Spinlock;
use crate::time::get_jiffies;
use crate::types::Jiffies;

/// Timer frequency (Hz) - how often timer interrupt fires
const TIMER_FREQUENCY: u64 = 1000; // 1000 Hz = 1ms intervals

/// Scheduler quantum (time slice) in timer ticks
const SCHEDULER_QUANTUM: u64 = 10; // 10ms default quantum

/// Timer statistics
#[derive(Debug, Clone)]
pub struct TimerStats {
	pub total_interrupts: u64,
	pub scheduler_invocations: u64,
	pub context_switches: u64,
	pub last_update: Jiffies,
}

/// Global timer state
pub struct TimerState {
	tick_count: AtomicU64,
	last_schedule_tick: AtomicU64,
	preemption_enabled: AtomicBool,
	stats: Spinlock<TimerStats>,
}

impl TimerState {
	const fn new() -> Self {
		Self {
			tick_count: AtomicU64::new(0),
			last_schedule_tick: AtomicU64::new(0),
			preemption_enabled: AtomicBool::new(true),
			stats: Spinlock::new(TimerStats {
				total_interrupts: 0,
				scheduler_invocations: 0,
				context_switches: 0,
				last_update: Jiffies(0),
			}),
		}
	}

	/// Handle timer interrupt
	pub fn handle_timer_interrupt(&self) {
		let current_tick = self.tick_count.fetch_add(1, Ordering::SeqCst);
		let last_schedule = self.last_schedule_tick.load(Ordering::SeqCst);

		// Update statistics
		{
			let mut stats = self.stats.lock();
			stats.total_interrupts += 1;
			stats.last_update = get_jiffies();
		}

		// Check if we should invoke the scheduler
		if self.preemption_enabled.load(Ordering::SeqCst) {
			let ticks_since_schedule = current_tick - last_schedule;

			if ticks_since_schedule >= SCHEDULER_QUANTUM {
				self.invoke_scheduler();
				self.last_schedule_tick
					.store(current_tick, Ordering::SeqCst);
			}
		}

		// Update current task runtime
		enhanced_scheduler::update_current_task_runtime(1);
	}

	/// Invoke the scheduler for preemptive multitasking
	fn invoke_scheduler(&self) {
		// Update statistics
		{
			let mut stats = self.stats.lock();
			stats.scheduler_invocations += 1;
		}

		// Get next task to run
		if let Some(next_tid) = enhanced_scheduler::schedule_next() {
			let current_tid = enhanced_scheduler::get_current_task();

			// Only switch if different task
			if current_tid != Some(next_tid) {
				if let Ok(()) = enhanced_scheduler::switch_to_task(next_tid) {
					// Update context switch statistics
					let mut stats = self.stats.lock();
					stats.context_switches += 1;

					// Perform actual context switch
					// This will save current CPU state and restore the state of the next task
					crate::scheduler::context_switch_to(next_tid);

					crate::info!(
						"Context switch: {:?} -> {:?}",
						current_tid,
						next_tid
					);
				}
			}
		}
	}

	/// Enable or disable preemption
	pub fn set_preemption_enabled(&self, enabled: bool) {
		self.preemption_enabled.store(enabled, Ordering::SeqCst);
		enhanced_scheduler::set_preemption_enabled(enabled);
	}

	/// Get timer statistics
	pub fn get_stats(&self) -> TimerStats {
		self.stats.lock().clone()
	}

	/// Get current tick count
	pub fn get_tick_count(&self) -> u64 {
		self.tick_count.load(Ordering::SeqCst)
	}

	/// Reset statistics
	pub fn reset_stats(&self) {
		let mut stats = self.stats.lock();
		stats.total_interrupts = 0;
		stats.scheduler_invocations = 0;
		stats.context_switches = 0;
		stats.last_update = get_jiffies();
	}

	/// Check if preemption is enabled
	pub fn is_preemption_enabled(&self) -> bool {
		self.preemption_enabled.load(Ordering::SeqCst)
	}
}

/// Timer interrupt counter
static TIMER_INTERRUPTS: AtomicU64 = AtomicU64::new(0);

/// Get timer interrupt count
pub fn get_timer_interrupts() -> u64 {
	TIMER_INTERRUPTS.load(Ordering::Relaxed)
}

/// Increment timer interrupt counter
pub fn increment_timer_interrupts() {
	TIMER_INTERRUPTS.fetch_add(1, Ordering::Relaxed);
}

/// Global timer state
static TIMER_STATE: TimerState = TimerState::new();

/// Initialize timer for preemptive scheduling
pub fn init_timer() -> crate::error::Result<()> {
	// Initialize the Programmable Interval Timer (PIT)
	init_pit(TIMER_FREQUENCY)?;

	// Enable timer interrupts
	crate::arch::x86_64::idt::register_timer_handler(timer_interrupt_handler);

	crate::info!(
		"Timer initialized for preemptive scheduling ({}Hz)",
		TIMER_FREQUENCY
	);
	Ok(())
}

/// Timer interrupt handler (called from IDT)
pub extern "C" fn timer_interrupt_handler() {
	TIMER_STATE.handle_timer_interrupt();
	increment_timer_interrupts();

	// Send EOI to PIC
	unsafe {
		crate::arch::x86_64::pic::send_eoi(0); // Timer is IRQ 0
	}
}

/// Initialize Programmable Interval Timer (PIT)
fn init_pit(frequency: u64) -> crate::error::Result<()> {
	use crate::arch::x86_64::port::Port;

	// PIT frequency is 1.193182 MHz
	const PIT_FREQUENCY: u64 = 1193182;

	// Calculate divisor for desired frequency
	let divisor = PIT_FREQUENCY / frequency;
	if divisor > 65535 {
		return Err(crate::error::Error::InvalidArgument);
	}

	unsafe {
		// Configure PIT channel 0 for periodic mode
		let mut cmd_port = Port::new(0x43);
		let mut data_port = Port::new(0x40);

		// Command: Channel 0, Access mode lobyte/hibyte, Mode 2 (rate generator)
		cmd_port.write(0x34u32);

		// Set divisor (low byte first, then high byte)
		data_port.write((divisor & 0xFF) as u32);
		data_port.write((divisor >> 8) as u32);
	}

	Ok(())
}

/// Get timer statistics
pub fn get_timer_stats() -> TimerStats {
	TIMER_STATE.get_stats()
}

/// Enable/disable preemptive scheduling
pub fn set_preemption_enabled(enabled: bool) {
	TIMER_STATE.set_preemption_enabled(enabled);
}

/// Get current timer tick count
pub fn get_timer_ticks() -> u64 {
	TIMER_STATE.get_tick_count()
}

/// Reset timer statistics
pub fn reset_timer_stats() {
	TIMER_STATE.reset_stats();
}

/// Sleep for specified number of timer ticks
pub fn sleep_ticks(ticks: u64) -> crate::error::Result<()> {
	enhanced_scheduler::sleep_current_task(ticks)
}

/// Yield current task to scheduler
pub fn yield_task() {
	TIMER_STATE.invoke_scheduler();
}

/// Handle timer tick - called from kernel loops for timing updates
pub fn handle_timer_tick() {
	// Update timer statistics
	TIMER_STATE.handle_timer_interrupt();

	// Trigger scheduler if preemption is enabled
	if TIMER_STATE.is_preemption_enabled() {
		yield_task();
	}
}
