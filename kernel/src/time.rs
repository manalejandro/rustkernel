// SPDX-License-Identifier: GPL-2.0

//! Time management compatible with Linux kernel

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::Result;
use crate::types::Jiffies; // Add Vec import

/// System clock frequency (Hz) - typically 1000 for 1ms ticks
pub const HZ: u64 = 1000;

/// Nanoseconds per second
pub const NSEC_PER_SEC: u64 = 1_000_000_000;

/// Nanoseconds per millisecond
pub const NSEC_PER_MSEC: u64 = 1_000_000;

/// Nanoseconds per microsecond
pub const NSEC_PER_USEC: u64 = 1_000;

/// Nanoseconds per jiffy
pub const NSEC_PER_JIFFY: u64 = NSEC_PER_SEC / HZ;

/// Global time counters
static JIFFIES_COUNTER: AtomicU64 = AtomicU64::new(0);
static BOOTTIME_NS: AtomicU64 = AtomicU64::new(0);

/// TSC (Time Stamp Counter) related globals
pub static TSC_FREQUENCY: AtomicU64 = AtomicU64::new(0);
static BOOT_TSC: AtomicU64 = AtomicU64::new(0);

/// Time structure - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpec {
	pub tv_sec: i64,
	pub tv_nsec: i64,
}

impl TimeSpec {
	pub const fn new(sec: i64, nsec: i64) -> Self {
		Self {
			tv_sec: sec,
			tv_nsec: nsec,
		}
	}

	pub fn zero() -> Self {
		Self::new(0, 0)
	}

	pub fn to_ns(&self) -> u64 {
		(self.tv_sec as u64 * NSEC_PER_SEC) + self.tv_nsec as u64
	}

	pub fn from_ns(ns: u64) -> Self {
		let sec = (ns / NSEC_PER_SEC) as i64;
		let nsec = (ns % NSEC_PER_SEC) as i64;
		Self::new(sec, nsec)
	}
}

/// Get current system time
pub fn get_current_time() -> TimeSpec {
	// In a real implementation, this would read from a real-time clock
	// For now, we'll use the boot time plus jiffies
	let boot_ns = BOOTTIME_NS.load(Ordering::Relaxed);
	let jiffies = get_jiffies();
	let current_ns = boot_ns + (jiffies * NSEC_PER_JIFFY);
	TimeSpec::from_ns(current_ns)
}

/// High resolution timer structure
#[derive(Debug, Clone)]
pub struct HrTimer {
	pub expires: TimeSpec,
	pub function: Option<fn()>,
	pub base: HrTimerBase,
}

/// Timer bases - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrTimerBase {
	Monotonic,
	Realtime,
	Boottime,
	Tai,
}

impl HrTimer {
	pub fn new(base: HrTimerBase) -> Self {
		Self {
			expires: TimeSpec::zero(),
			function: None,
			base,
		}
	}

	pub fn set_expires(&mut self, expires: TimeSpec) {
		self.expires = expires;
	}

	pub fn set_function(&mut self, function: fn()) {
		self.function = Some(function);
	}

	pub fn is_expired(&self) -> bool {
		let now = match self.base {
			HrTimerBase::Monotonic => monotonic_time(),
			HrTimerBase::Realtime => get_realtime(),
			HrTimerBase::Boottime => get_boottime(),
			HrTimerBase::Tai => get_realtime(), // Simplified
		};
		now >= self.expires
	}
}

/// Initialize time management
pub fn init() -> Result<()> {
	// Initialize system clocks
	let boot_time = read_hardware_clock();
	BOOTTIME_NS.store(boot_time, Ordering::Relaxed);

	// Set up timer interrupts through our timer module
	crate::timer::init_timer()?;

	// Initialize high-resolution timers
	init_high_res_timers()?;

	crate::info!("Time management initialized, boot time: {} ns", boot_time);
	Ok(())
}

/// Initialize high-resolution timers
fn init_high_res_timers() -> Result<()> {
	// Initialize TSC (Time Stamp Counter) frequency detection
	let tsc_freq = detect_tsc_frequency();
	TSC_FREQUENCY.store(tsc_freq, Ordering::Relaxed);

	// Store initial TSC value for relative timing
	BOOT_TSC.store(read_tsc(), Ordering::Relaxed);

	crate::info!(
		"High-resolution timers initialized, TSC frequency: {} Hz",
		tsc_freq
	);
	Ok(())
}

/// Detect TSC frequency using PIT calibration
fn detect_tsc_frequency() -> u64 {
	// Calibrate TSC against PIT (Programmable Interval Timer)
	// This is a simplified implementation

	unsafe {
		// Set PIT to mode 2 (rate generator) with a known frequency
		crate::arch::x86_64::port::outb(0x43, 0x34); // Channel 0, mode 2

		// Set frequency to ~1193 Hz (divisor = 1000)
		let divisor = 1000u16;
		crate::arch::x86_64::port::outb(0x40, (divisor & 0xFF) as u8);
		crate::arch::x86_64::port::outb(0x40, (divisor >> 8) as u8);

		// Read initial TSC
		let tsc_start = read_tsc();

		// Wait for a PIT tick (simplified timing)
		let mut last_pit = read_pit_count();
		let mut pit_ticks = 0;

		while pit_ticks < 10 {
			// Wait for ~10ms
			let current_pit = read_pit_count();
			if current_pit != last_pit {
				pit_ticks += 1;
				last_pit = current_pit;
			}
		}

		// Read final TSC
		let tsc_end = read_tsc();
		let tsc_delta = tsc_end - tsc_start;

		// Calculate frequency (rough approximation)
		// 10 PIT ticks at ~1193 Hz = ~8.4ms
		let frequency = (tsc_delta * 1193) / 10;

		// Reasonable bounds checking
		if frequency < 100_000_000 || frequency > 10_000_000_000 {
			// Default to 2.4 GHz if calibration seems wrong
			2_400_000_000
		} else {
			frequency
		}
	}
}

/// Read PIT counter value
unsafe fn read_pit_count() -> u16 {
	crate::arch::x86_64::port::outb(0x43, 0x00); // Latch counter 0
	let low = crate::arch::x86_64::port::inb(0x40) as u16;
	let high = crate::arch::x86_64::port::inb(0x40) as u16;
	(high << 8) | low
}

/// Initialize time management subsystem
pub fn init_time() -> Result<()> {
	// Initialize the timer wheel
	let _timer_wheel = get_timer_wheel();

	// Set initial boot time (in a real implementation, this would read from RTC)
	BOOTTIME_NS.store(0, Ordering::Relaxed);

	// Reset jiffies counter
	JIFFIES_COUNTER.store(0, Ordering::Relaxed);

	crate::info!("Time management initialized");
	Ok(())
}

/// Read hardware clock implementation
fn read_hardware_clock() -> u64 {
	// Read from CMOS RTC (Real Time Clock)
	// This is a simplified implementation
	unsafe {
		// Disable NMI and read seconds
		crate::arch::x86_64::port::outb(0x70, 0x00);
		let seconds = crate::arch::x86_64::port::inb(0x71);

		// Read minutes
		crate::arch::x86_64::port::outb(0x70, 0x02);
		let minutes = crate::arch::x86_64::port::inb(0x71);

		// Read hours
		crate::arch::x86_64::port::outb(0x70, 0x04);
		let hours = crate::arch::x86_64::port::inb(0x71);

		// Convert BCD to binary if needed (simplified)
		let sec = bcd_to_bin(seconds);
		let min = bcd_to_bin(minutes);
		let hr = bcd_to_bin(hours);

		// Convert to nanoseconds since epoch (simplified calculation)
		let total_seconds = (hr as u64 * 3600) + (min as u64 * 60) + sec as u64;
		total_seconds * 1_000_000_000 // Convert to nanoseconds
	}
}

/// Convert BCD to binary
fn bcd_to_bin(bcd: u8) -> u8 {
	((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Get current jiffies count
pub fn get_jiffies() -> Jiffies {
	Jiffies(JIFFIES_COUNTER.load(Ordering::Relaxed))
}

/// Increment jiffies counter (called from timer interrupt)
pub fn update_jiffies() {
	JIFFIES_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Get current time in nanoseconds since boot
pub fn get_time_ns() -> u64 {
	// Use TSC for high-resolution timing
	let tsc_freq = TSC_FREQUENCY.load(Ordering::Relaxed);
	if tsc_freq > 0 {
		let tsc = read_tsc();
		let boot_tsc = BOOT_TSC.load(Ordering::Relaxed);
		if tsc >= boot_tsc {
			((tsc - boot_tsc) * 1_000_000_000) / tsc_freq
		} else {
			// Handle TSC overflow (rare)
			get_jiffies().0 * NSEC_PER_JIFFY
		}
	} else {
		// Fallback to jiffies-based timing
		get_jiffies().0 * NSEC_PER_JIFFY
	}
}

/// Get high resolution time
pub fn ktime_get() -> TimeSpec {
	let ns = get_time_ns();
	TimeSpec {
		tv_sec: (ns / 1_000_000_000) as i64,
		tv_nsec: (ns % 1_000_000_000) as i64,
	}
}

/// Read Time Stamp Counter
fn read_tsc() -> u64 {
	unsafe {
		let low: u32;
		let high: u32;
		core::arch::asm!(
		    "rdtsc",
		    out("eax") low,
		    out("edx") high,
		    options(nomem, nostack, preserves_flags)
		);
		((high as u64) << 32) | (low as u64)
	}
}

/// Get monotonic time (time since boot)
pub fn monotonic_time() -> TimeSpec {
	let jiffies = get_jiffies();
	let ns = jiffies.0 * NSEC_PER_JIFFY;
	TimeSpec::from_ns(ns)
}

/// Get boot time
pub fn get_boottime() -> TimeSpec {
	let boot_ns = BOOTTIME_NS.load(Ordering::Relaxed);
	let current_ns = get_time_ns();
	TimeSpec::from_ns(boot_ns + current_ns)
}

/// Get real time (wall clock time)
pub fn get_realtime() -> TimeSpec {
	get_boottime()
}

/// Convert nanoseconds to jiffies
pub fn ns_to_jiffies(ns: u64) -> Jiffies {
	Jiffies(ns / NSEC_PER_JIFFY)
}

/// Convert jiffies to nanoseconds
pub fn jiffies_to_ns(jiffies: Jiffies) -> u64 {
	jiffies.0 * NSEC_PER_JIFFY
}

/// Convert milliseconds to jiffies
pub fn msecs_to_jiffies(ms: u64) -> Jiffies {
	ns_to_jiffies(ms * NSEC_PER_MSEC)
}

/// Convert jiffies to milliseconds
pub fn jiffies_to_msecs(jiffies: Jiffies) -> u64 {
	jiffies_to_ns(jiffies) / NSEC_PER_MSEC
}

/// Convert microseconds to jiffies
pub fn usecs_to_jiffies(us: u64) -> Jiffies {
	ns_to_jiffies(us * NSEC_PER_USEC)
}

/// Convert jiffies to microseconds
pub fn jiffies_to_usecs(jiffies: Jiffies) -> u64 {
	jiffies_to_ns(jiffies) / NSEC_PER_USEC
}

/// Sleep functions - Linux compatible
pub fn msleep(ms: u64) {
	let target_jiffies = get_jiffies().0 + msecs_to_jiffies(ms).0;
	while get_jiffies().0 < target_jiffies {
		crate::scheduler::yield_now();
	}
}

pub fn usleep_range(min_us: u64, max_us: u64) {
	let us = (min_us + max_us) / 2; // Use average
	let target_jiffies = get_jiffies().0 + usecs_to_jiffies(us).0;
	while get_jiffies().0 < target_jiffies {
		crate::scheduler::yield_now();
	}
}

pub fn ndelay(ns: u64) {
	// Busy wait for nanoseconds (not recommended for long delays)
	let start = get_time_ns();
	while get_time_ns() - start < ns {
		core::hint::spin_loop();
	}
}

pub fn udelay(us: u64) {
	ndelay(us * NSEC_PER_USEC);
}

pub fn mdelay(ms: u64) {
	ndelay(ms * NSEC_PER_MSEC);
}

/// Timer wheel for managing timers
#[derive(Debug)]
pub struct TimerWheel {
	levels: [Vec<HrTimer>; 8], // Multiple levels for different time ranges
	current_jiffies: u64,
}

impl TimerWheel {
	pub fn new() -> Self {
		const EMPTY_VEC: Vec<HrTimer> = Vec::new();
		Self {
			levels: [EMPTY_VEC; 8],
			current_jiffies: 0,
		}
	}

	pub fn add_timer(&mut self, timer: HrTimer) {
		let now_ns = get_time_ns();
		let expires_ns = timer.expires.to_ns();

		// If already expired or expires very soon, put in level 0
		if expires_ns <= now_ns {
			self.levels[0].push(timer);
			return;
		}

		let delta_ns = expires_ns - now_ns;
		let delta_jiffies = delta_ns / NSEC_PER_JIFFY;

		// Determine level based on delta
		// Level 0: Immediate to ~256ms
		// Level 1: ~256ms to ~16s
		// Level 2: ~16s to ~17m
		// Level 3: ~17m to ~18h
		// Level 4+: Far future
		let level = if delta_jiffies < 256 {
			0
		} else if delta_jiffies < 256 * 64 {
			1
		} else if delta_jiffies < 256 * 64 * 64 {
			2
		} else if delta_jiffies < 256 * 64 * 64 * 64 {
			3
		} else {
			4
		};

		let level = core::cmp::min(level, 7);
		self.levels[level].push(timer);
	}

	pub fn run_timers(&mut self) {
		self.current_jiffies = get_jiffies().0;

		// Check and run expired timers
		for level in &mut self.levels {
			level.retain(|timer| {
				if timer.is_expired() {
					if let Some(function) = timer.function {
						function();
					}
					false // Remove expired timer
				} else {
					true // Keep timer
				}
			});
		}
	}
}

use core::sync::atomic::AtomicBool;

/// Global timer wheel
use crate::sync::Spinlock;

static TIMER_WHEEL_INIT: AtomicBool = AtomicBool::new(false);
static mut TIMER_WHEEL_STORAGE: Option<Spinlock<TimerWheel>> = None;

fn get_timer_wheel() -> &'static Spinlock<TimerWheel> {
	if !TIMER_WHEEL_INIT.load(Ordering::Acquire) {
		// Initialize timer wheel
		let wheel = TimerWheel::new();
		unsafe {
			TIMER_WHEEL_STORAGE = Some(Spinlock::new(wheel));
		}
		TIMER_WHEEL_INIT.store(true, Ordering::Release);
	}

	#[allow(static_mut_refs)]
	unsafe {
		TIMER_WHEEL_STORAGE.as_ref().unwrap()
	}
}

/// Add a timer to the system
pub fn add_timer(timer: HrTimer) {
	let timer_wheel = get_timer_wheel();
	let mut wheel = timer_wheel.lock();
	wheel.add_timer(timer);
}

/// Run expired timers (called from timer interrupt)
pub fn run_timers() {
	let timer_wheel = get_timer_wheel();
	let mut wheel = timer_wheel.lock();
	wheel.run_timers();
}

/// Timer interrupt handler
pub fn timer_interrupt() {
	// Update jiffies
	update_jiffies();

	// Run expired timers
	run_timers();

	// Update scheduler tick
	crate::scheduler::scheduler_tick();
}
