// SPDX-License-Identifier: GPL-2.0

//! Synchronization primitives

// Re-export common synchronization types
pub use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub use spin::Mutex;
pub use spin::RwLock;

/// Spinlock implementation
pub struct Spinlock<T> {
	locked: AtomicBool,
	data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Spinlock<T> {}
unsafe impl<T: Send> Send for Spinlock<T> {}

impl<T> Spinlock<T> {
	pub const fn new(data: T) -> Self {
		Self {
			locked: AtomicBool::new(false),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> SpinlockGuard<'_, T> {
		while self
			.locked
			.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
			.is_err()
		{
			// Busy wait
			while self.locked.load(Ordering::Relaxed) {
				core::hint::spin_loop();
			}
		}

		SpinlockGuard { lock: self }
	}

	pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
		if self.locked
			.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
		{
			Some(SpinlockGuard { lock: self })
		} else {
			None
		}
	}
}

pub struct SpinlockGuard<'a, T> {
	lock: &'a Spinlock<T>,
}

impl<T> Deref for SpinlockGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.lock.data.get() }
	}
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.lock.data.get() }
	}
}

impl<T> Drop for SpinlockGuard<'_, T> {
	fn drop(&mut self) {
		self.lock.locked.store(false, Ordering::Release);
	}
}

// Note: We use spin::Mutex and spin::RwLock for actual implementations
// The Spinlock above is for cases where we need a simple spinlock specifically
