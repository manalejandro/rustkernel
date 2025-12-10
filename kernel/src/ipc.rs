// SPDX-License-Identifier: GPL-2.0

//! Advanced Inter-Process Communication (IPC) system

use alloc::{
	collections::{BTreeMap, VecDeque},
	string::String,
	vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::types::Tid;

/// IPC message types
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
	Data,
	Signal,
	Request,
	Response,
	Broadcast,
	Priority,
}

/// IPC message structure
#[derive(Debug, Clone)]
pub struct Message {
	pub id: u64,
	pub sender: Tid,
	pub recipient: Tid,
	pub msg_type: MessageType,
	pub data: Vec<u8>,
	pub timestamp: u64,
	pub priority: u8,
}

/// Message queue for a process
#[derive(Debug)]
pub struct MessageQueue {
	pub messages: VecDeque<Message>,
	pub max_size: usize,
	pub blocked_senders: Vec<Tid>,
	pub blocked_receivers: Vec<Tid>,
}

impl MessageQueue {
	pub fn new(max_size: usize) -> Self {
		Self {
			messages: VecDeque::new(),
			max_size,
			blocked_senders: Vec::new(),
			blocked_receivers: Vec::new(),
		}
	}

	pub fn is_full(&self) -> bool {
		self.messages.len() >= self.max_size
	}

	pub fn is_empty(&self) -> bool {
		self.messages.is_empty()
	}
}

/// Semaphore for synchronization
#[derive(Debug)]
pub struct Semaphore {
	pub value: i32,
	pub waiting_tasks: VecDeque<Tid>,
}

impl Semaphore {
	pub fn new(initial_value: i32) -> Self {
		Self {
			value: initial_value,
			waiting_tasks: VecDeque::new(),
		}
	}
}

/// Shared memory region
#[derive(Debug)]
pub struct SharedMemory {
	pub id: u64,
	pub size: usize,
	pub address: usize,
	pub owners: Vec<Tid>,
	pub permissions: u32,
	pub ref_count: usize,
}

/// IPC statistics
#[derive(Debug, Default)]
pub struct IpcStats {
	pub messages_sent: AtomicU64,
	pub messages_received: AtomicU64,
	pub semaphore_operations: AtomicU64,
	pub shared_memory_attachments: AtomicU64,
	pub pipe_operations: AtomicU64,
}

/// Advanced IPC manager
pub struct IpcManager {
	message_queues: Spinlock<BTreeMap<Tid, MessageQueue>>,
	semaphores: Spinlock<BTreeMap<u64, Semaphore>>,
	shared_memory: Spinlock<BTreeMap<u64, SharedMemory>>,
	pipes: Spinlock<BTreeMap<u64, VecDeque<u8>>>,
	next_message_id: AtomicU64,
	next_semaphore_id: AtomicU64,
	next_shm_id: AtomicU64,
	next_pipe_id: AtomicU64,
	stats: IpcStats,
}

impl IpcManager {
	pub const fn new() -> Self {
		Self {
			message_queues: Spinlock::new(BTreeMap::new()),
			semaphores: Spinlock::new(BTreeMap::new()),
			shared_memory: Spinlock::new(BTreeMap::new()),
			pipes: Spinlock::new(BTreeMap::new()),
			next_message_id: AtomicU64::new(1),
			next_semaphore_id: AtomicU64::new(1),
			next_shm_id: AtomicU64::new(1),
			next_pipe_id: AtomicU64::new(1),
			stats: IpcStats {
				messages_sent: AtomicU64::new(0),
				messages_received: AtomicU64::new(0),
				semaphore_operations: AtomicU64::new(0),
				shared_memory_attachments: AtomicU64::new(0),
				pipe_operations: AtomicU64::new(0),
			},
		}
	}

	/// Create message queue for a process
	pub fn create_message_queue(&self, tid: Tid, max_size: usize) -> Result<()> {
		let mut queues = self.message_queues.lock();
		if queues.contains_key(&tid) {
			return Err(Error::AlreadyExists);
		}
		queues.insert(tid, MessageQueue::new(max_size));
		Ok(())
	}

	/// Send message to another process
	pub fn send_message(
		&self,
		sender: Tid,
		recipient: Tid,
		msg_type: MessageType,
		data: Vec<u8>,
		priority: u8,
	) -> Result<u64> {
		let message_id = self.next_message_id.fetch_add(1, Ordering::Relaxed);
		let message = Message {
			id: message_id,
			sender,
			recipient,
			msg_type,
			data,
			timestamp: crate::time::get_jiffies().0,
			priority,
		};

		let mut queues = self.message_queues.lock();
		match queues.get_mut(&recipient) {
			Some(queue) => {
				if queue.is_full() {
					// Queue is full, block sender or return error
					return Err(Error::ResourceBusy);
				}

				// Insert message in priority order
				let insert_pos = queue
					.messages
					.iter()
					.position(|m| m.priority < priority)
					.unwrap_or(queue.messages.len());
				queue.messages.insert(insert_pos, message);

				self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
				Ok(message_id)
			}
			None => Err(Error::NotFound),
		}
	}

	/// Receive message from queue
	pub fn receive_message(&self, tid: Tid) -> Result<Option<Message>> {
		let mut queues = self.message_queues.lock();
		match queues.get_mut(&tid) {
			Some(queue) => {
				if let Some(message) = queue.messages.pop_front() {
					self.stats
						.messages_received
						.fetch_add(1, Ordering::Relaxed);
					Ok(Some(message))
				} else {
					Ok(None)
				}
			}
			None => Err(Error::NotFound),
		}
	}

	/// Create semaphore
	pub fn create_semaphore(&self, initial_value: i32) -> Result<u64> {
		let sem_id = self.next_semaphore_id.fetch_add(1, Ordering::Relaxed);
		let mut semaphores = self.semaphores.lock();
		semaphores.insert(sem_id, Semaphore::new(initial_value));
		Ok(sem_id)
	}

	/// Wait on semaphore (P operation)
	pub fn semaphore_wait(&self, sem_id: u64, tid: Tid) -> Result<bool> {
		let mut semaphores = self.semaphores.lock();
		match semaphores.get_mut(&sem_id) {
			Some(semaphore) => {
				if semaphore.value > 0 {
					semaphore.value -= 1;
					self.stats
						.semaphore_operations
						.fetch_add(1, Ordering::Relaxed);
					Ok(true) // Acquired immediately
				} else {
					semaphore.waiting_tasks.push_back(tid);
					Ok(false) // Would block
				}
			}
			None => Err(Error::NotFound),
		}
	}

	/// Signal semaphore (V operation)
	pub fn semaphore_signal(&self, sem_id: u64) -> Result<Option<Tid>> {
		let mut semaphores = self.semaphores.lock();
		match semaphores.get_mut(&sem_id) {
			Some(semaphore) => {
				semaphore.value += 1;
				let woken_task = semaphore.waiting_tasks.pop_front();
				if woken_task.is_some() {
					semaphore.value -= 1; // Task will consume the signal
				}
				self.stats
					.semaphore_operations
					.fetch_add(1, Ordering::Relaxed);
				Ok(woken_task)
			}
			None => Err(Error::NotFound),
		}
	}

	/// Create shared memory region
	pub fn create_shared_memory(&self, size: usize, permissions: u32) -> Result<u64> {
		let shm_id = self.next_shm_id.fetch_add(1, Ordering::Relaxed);

		// Allocate memory (simplified - in reality would use page allocator)
		let address = crate::memory::kmalloc::kmalloc(size)?;

		let shm = SharedMemory {
			id: shm_id,
			size,
			address: address as usize,
			owners: Vec::new(),
			permissions,
			ref_count: 0,
		};

		let mut shared_memory = self.shared_memory.lock();
		shared_memory.insert(shm_id, shm);
		Ok(shm_id)
	}

	/// Attach to shared memory
	pub fn attach_shared_memory(&self, shm_id: u64, tid: Tid) -> Result<usize> {
		let mut shared_memory = self.shared_memory.lock();
		match shared_memory.get_mut(&shm_id) {
			Some(shm) => {
				if !shm.owners.contains(&tid) {
					shm.owners.push(tid);
					shm.ref_count += 1;
				}
				self.stats
					.shared_memory_attachments
					.fetch_add(1, Ordering::Relaxed);
				Ok(shm.address)
			}
			None => Err(Error::NotFound),
		}
	}

	/// Create pipe
	pub fn create_pipe(&self) -> Result<u64> {
		let pipe_id = self.next_pipe_id.fetch_add(1, Ordering::Relaxed);
		let mut pipes = self.pipes.lock();
		pipes.insert(pipe_id, VecDeque::new());
		Ok(pipe_id)
	}

	/// Write to pipe
	pub fn pipe_write(&self, pipe_id: u64, data: &[u8]) -> Result<usize> {
		let mut pipes = self.pipes.lock();
		match pipes.get_mut(&pipe_id) {
			Some(pipe) => {
				pipe.extend(data.iter().cloned());
				self.stats.pipe_operations.fetch_add(1, Ordering::Relaxed);
				Ok(data.len())
			}
			None => Err(Error::NotFound),
		}
	}

	/// Read from pipe  
	pub fn pipe_read(&self, pipe_id: u64, buffer: &mut [u8]) -> Result<usize> {
		let mut pipes = self.pipes.lock();
		match pipes.get_mut(&pipe_id) {
			Some(pipe) => {
				let read_len = buffer.len().min(pipe.len());
				for i in 0..read_len {
					buffer[i] = pipe.pop_front().unwrap();
				}
				self.stats.pipe_operations.fetch_add(1, Ordering::Relaxed);
				Ok(read_len)
			}
			None => Err(Error::NotFound),
		}
	}

	/// Get IPC statistics
	pub fn get_stats(&self) -> IpcStatsSnapshot {
		IpcStatsSnapshot {
			messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
			messages_received: self.stats.messages_received.load(Ordering::Relaxed),
			semaphore_operations: self
				.stats
				.semaphore_operations
				.load(Ordering::Relaxed),
			shared_memory_attachments: self
				.stats
				.shared_memory_attachments
				.load(Ordering::Relaxed),
			pipe_operations: self.stats.pipe_operations.load(Ordering::Relaxed),
			active_queues: self.message_queues.lock().len(),
			active_semaphores: self.semaphores.lock().len(),
			active_shared_memory: self.shared_memory.lock().len(),
			active_pipes: self.pipes.lock().len(),
		}
	}

	/// Cleanup resources for a terminated process
	pub fn cleanup_process(&self, tid: Tid) -> Result<()> {
		// Remove message queue
		self.message_queues.lock().remove(&tid);

		// Remove from semaphore waiting lists
		let mut semaphores = self.semaphores.lock();
		for semaphore in semaphores.values_mut() {
			semaphore.waiting_tasks.retain(|&t| t != tid);
		}

		// Detach from shared memory
		let mut shared_memory = self.shared_memory.lock();
		let mut to_remove = Vec::new();
		for (id, shm) in shared_memory.iter_mut() {
			if let Some(pos) = shm.owners.iter().position(|&t| t == tid) {
				shm.owners.remove(pos);
				shm.ref_count -= 1;
				if shm.ref_count == 0 {
					to_remove.push(*id);
				}
			}
		}

		// Free unused shared memory
		for id in to_remove {
			if let Some(shm) = shared_memory.remove(&id) {
				unsafe {
					crate::memory::kmalloc::kfree(shm.address as *mut u8);
				}
			}
		}

		Ok(())
	}
}

/// IPC statistics snapshot
#[derive(Debug, Clone)]
pub struct IpcStatsSnapshot {
	pub messages_sent: u64,
	pub messages_received: u64,
	pub semaphore_operations: u64,
	pub shared_memory_attachments: u64,
	pub pipe_operations: u64,
	pub active_queues: usize,
	pub active_semaphores: usize,
	pub active_shared_memory: usize,
	pub active_pipes: usize,
}

/// Global IPC manager
static IPC_MANAGER: IpcManager = IpcManager::new();

/// Initialize IPC system
pub fn init_ipc() -> Result<()> {
	crate::info!("IPC system initialized");
	Ok(())
}

/// Create message queue for process
pub fn create_message_queue(tid: Tid, max_size: usize) -> Result<()> {
	IPC_MANAGER.create_message_queue(tid, max_size)
}

/// Send message
pub fn send_message(
	sender: Tid,
	recipient: Tid,
	msg_type: MessageType,
	data: Vec<u8>,
	priority: u8,
) -> Result<u64> {
	IPC_MANAGER.send_message(sender, recipient, msg_type, data, priority)
}

/// Receive message
pub fn receive_message(tid: Tid) -> Result<Option<Message>> {
	IPC_MANAGER.receive_message(tid)
}

/// Create semaphore
pub fn create_semaphore(initial_value: i32) -> Result<u64> {
	IPC_MANAGER.create_semaphore(initial_value)
}

/// Wait on semaphore
pub fn semaphore_wait(sem_id: u64, tid: Tid) -> Result<bool> {
	IPC_MANAGER.semaphore_wait(sem_id, tid)
}

/// Signal semaphore
pub fn semaphore_signal(sem_id: u64) -> Result<Option<Tid>> {
	IPC_MANAGER.semaphore_signal(sem_id)
}

/// Create shared memory
pub fn create_shared_memory(size: usize, permissions: u32) -> Result<u64> {
	IPC_MANAGER.create_shared_memory(size, permissions)
}

/// Attach to shared memory
pub fn attach_shared_memory(shm_id: u64, tid: Tid) -> Result<usize> {
	IPC_MANAGER.attach_shared_memory(shm_id, tid)
}

/// Create pipe
pub fn create_pipe() -> Result<u64> {
	IPC_MANAGER.create_pipe()
}

/// Write to pipe
pub fn pipe_write(pipe_id: u64, data: &[u8]) -> Result<usize> {
	IPC_MANAGER.pipe_write(pipe_id, data)
}

/// Read from pipe
pub fn pipe_read(pipe_id: u64, buffer: &mut [u8]) -> Result<usize> {
	IPC_MANAGER.pipe_read(pipe_id, buffer)
}

/// Get IPC statistics
pub fn get_ipc_stats() -> IpcStatsSnapshot {
	IPC_MANAGER.get_stats()
}

/// Cleanup process IPC resources
pub fn cleanup_process_ipc(tid: Tid) -> Result<()> {
	IPC_MANAGER.cleanup_process(tid)
}
