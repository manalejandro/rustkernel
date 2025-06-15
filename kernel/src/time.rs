// SPDX-License-Identifier: GPL-2.0

//! Time management compatible with Linux kernel

use crate::error::Result;
use crate::types::Jiffies;
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;  // Add Vec import

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

/// Time structure - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpec {
    pub sec: i64,
    pub nsec: i64,
}

impl TimeSpec {
    pub const fn new(sec: i64, nsec: i64) -> Self {
        Self { sec, nsec }
    }
    
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
    
    pub fn to_ns(&self) -> u64 {
        (self.sec as u64 * NSEC_PER_SEC) + self.nsec as u64
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
            HrTimerBase::Monotonic => get_monotonic_time(),
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
    
    // TODO: Set up timer interrupts
    // TODO: Initialize high-resolution timers
    
    crate::info!("Time management initialized, boot time: {} ns", boot_time);
    Ok(())
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

/// Read hardware clock (placeholder)
fn read_hardware_clock() -> u64 {
    // TODO: Read from actual hardware clock (RTC, TSC, etc.)
    // For now, return a fixed value
    1609459200_000_000_000 // 2021-01-01 00:00:00 UTC in nanoseconds
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
    // TODO: Read from high-resolution clock source (TSC, etc.)
    // For now, use jiffies-based approximation
    get_jiffies().0 * NSEC_PER_JIFFY
}

/// Get monotonic time (time since boot)
pub fn get_monotonic_time() -> TimeSpec {
    TimeSpec::from_ns(get_time_ns())
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
        // TODO: Add timer to appropriate level based on expiry time
        let level = 0; // Simplified
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

/// Global timer wheel
use crate::sync::Spinlock;
use core::sync::atomic::AtomicBool;

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
