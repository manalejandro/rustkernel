// SPDX-License-Identifier: GPL-2.0

//! Interrupt handling compatible with Linux kernel

use alloc::{boxed::Box, collections::BTreeMap}; // Add Box import
use core::arch::asm;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;

/// Global interrupt counter
static INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);

/// Get total interrupt count
pub fn get_interrupt_count() -> u64 {
	INTERRUPT_COUNT.load(Ordering::Relaxed)
}

/// Increment interrupt counter
pub fn increment_interrupt_count() {
	INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// IRQ flags - compatible with Linux kernel
pub mod irq_flags {
	pub const IRQF_SHARED: u32 = 0x00000080;
	pub const IRQF_TRIGGER_NONE: u32 = 0x00000000;
	pub const IRQF_TRIGGER_RISING: u32 = 0x00000001;
	pub const IRQF_TRIGGER_FALLING: u32 = 0x00000002;
	pub const IRQF_TRIGGER_HIGH: u32 = 0x00000004;
	pub const IRQF_TRIGGER_LOW: u32 = 0x00000008;
	pub const IRQF_ONESHOT: u32 = 0x00002000;
	pub const IRQF_NO_SUSPEND: u32 = 0x00004000;
	pub const IRQF_FORCE_RESUME: u32 = 0x00008000;
	pub const IRQF_NO_THREAD: u32 = 0x00010000;
	pub const IRQF_EARLY_RESUME: u32 = 0x00020000;
	pub const IRQF_COND_SUSPEND: u32 = 0x00040000;
}

/// Interrupt return values - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqReturn {
	None,       // IRQ_NONE
	Handled,    // IRQ_HANDLED
	WakeThread, // IRQ_WAKE_THREAD
}

impl fmt::Display for IrqReturn {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			IrqReturn::None => write!(f, "IRQ_NONE"),
			IrqReturn::Handled => write!(f, "IRQ_HANDLED"),
			IrqReturn::WakeThread => write!(f, "IRQ_WAKE_THREAD"),
		}
	}
}

/// Interrupt handler function type - Linux compatible
pub type IrqHandler = fn(irq: u32, dev_id: *mut u8) -> IrqReturn;

/// A wrapper for device pointer that can be safely shared between threads
/// In kernel code, we know the device pointer is valid for the lifetime of the
/// driver
#[derive(Debug)]
pub struct DevicePointer(*mut u8);

impl DevicePointer {
	pub fn new(ptr: *mut u8) -> Self {
		Self(ptr)
	}

	pub fn as_ptr(&self) -> *mut u8 {
		self.0
	}
}

// SAFETY: In kernel code, device pointers are managed by the kernel
// and are valid for the lifetime of the driver registration
unsafe impl Send for DevicePointer {}
unsafe impl Sync for DevicePointer {}

/// Interrupt action structure - similar to Linux irqaction
#[derive(Debug)]
pub struct IrqAction {
	pub handler: IrqHandler,
	pub flags: u32,
	pub name: &'static str,
	pub dev_id: DevicePointer,
	pub next: Option<Box<IrqAction>>,
}

impl IrqAction {
	pub fn new(handler: IrqHandler, flags: u32, name: &'static str, dev_id: *mut u8) -> Self {
		Self {
			handler,
			flags,
			name,
			dev_id: DevicePointer::new(dev_id),
			next: None,
		}
	}
}

/// Interrupt descriptor - similar to Linux irq_desc
#[derive(Debug)]
pub struct IrqDescriptor {
	pub irq: u32,
	pub action: Option<Box<IrqAction>>,
	pub depth: u32,          // nesting depth for disable/enable
	pub wake_depth: u32,     // wake nesting depth
	pub irq_count: u64,      // number of interrupts
	pub irqs_unhandled: u64, // number of unhandled interrupts
	pub name: &'static str,
	pub status: u32,
}

impl IrqDescriptor {
	pub fn new(irq: u32, name: &'static str) -> Self {
		Self {
			irq,
			action: None,
			depth: 1, // starts disabled
			wake_depth: 0,
			irq_count: 0,
			irqs_unhandled: 0,
			name,
			status: 0,
		}
	}

	pub fn is_enabled(&self) -> bool {
		self.depth == 0
	}

	pub fn enable(&mut self) {
		if self.depth > 0 {
			self.depth -= 1;
		}
	}

	pub fn disable(&mut self) {
		self.depth += 1;
	}
}

/// Global interrupt subsystem
static INTERRUPT_SUBSYSTEM: Spinlock<InterruptSubsystem> = Spinlock::new(InterruptSubsystem::new());

/// Interrupt subsystem state
struct InterruptSubsystem {
	descriptors: BTreeMap<u32, IrqDescriptor>,
	enabled: bool,
}

impl InterruptSubsystem {
	const fn new() -> Self {
		Self {
			descriptors: BTreeMap::new(),
			enabled: false,
		}
	}

	fn add_descriptor(&mut self, desc: IrqDescriptor) {
		let irq = desc.irq;
		self.descriptors.insert(irq, desc);
	}

	fn get_descriptor_mut(&mut self, irq: u32) -> Option<&mut IrqDescriptor> {
		self.descriptors.get_mut(&irq)
	}

	#[allow(dead_code)]
	fn get_descriptor(&self, irq: u32) -> Option<&IrqDescriptor> {
		self.descriptors.get(&irq)
	}
}

/// Initialize interrupt handling
pub fn init() -> Result<()> {
	let mut subsystem = INTERRUPT_SUBSYSTEM.lock();

	// Initialize architecture-specific interrupt handling
	crate::arch::x86_64::gdt::init();
	crate::arch::x86_64::idt::init();

	// Set up standard x86 interrupt vectors
	init_standard_interrupts(&mut subsystem)?;

	// Initialize interrupt controller (PIC/APIC)
	init_interrupt_controller()?;

	subsystem.enabled = true;
	crate::info!("Interrupt subsystem initialized");

	Ok(())
}

/// Initialize standard x86 interrupts
fn init_standard_interrupts(subsystem: &mut InterruptSubsystem) -> Result<()> {
	// Timer interrupt (IRQ 0)
	let timer_desc = IrqDescriptor::new(0, "timer");
	subsystem.add_descriptor(timer_desc);

	// Keyboard interrupt (IRQ 1)
	let keyboard_desc = IrqDescriptor::new(1, "keyboard");
	subsystem.add_descriptor(keyboard_desc);

	// Cascade for slave PIC (IRQ 2)
	let cascade_desc = IrqDescriptor::new(2, "cascade");
	subsystem.add_descriptor(cascade_desc);

	// Serial port 2/4 (IRQ 3)
	let serial_desc = IrqDescriptor::new(3, "serial");
	subsystem.add_descriptor(serial_desc);

	// Serial port 1/3 (IRQ 4)
	let serial2_desc = IrqDescriptor::new(4, "serial");
	subsystem.add_descriptor(serial2_desc);

	// Parallel port (IRQ 7)
	let parallel_desc = IrqDescriptor::new(7, "parallel");
	subsystem.add_descriptor(parallel_desc);

	// Real-time clock (IRQ 8)
	let rtc_desc = IrqDescriptor::new(8, "rtc");
	subsystem.add_descriptor(rtc_desc);

	// Mouse (IRQ 12)
	let mouse_desc = IrqDescriptor::new(12, "mouse");
	subsystem.add_descriptor(mouse_desc);

	// IDE primary (IRQ 14)
	let ide1_desc = IrqDescriptor::new(14, "ide");
	subsystem.add_descriptor(ide1_desc);

	// IDE secondary (IRQ 15)
	let ide2_desc = IrqDescriptor::new(15, "ide");
	subsystem.add_descriptor(ide2_desc);

	Ok(())
}

/// Initialize interrupt controller
fn init_interrupt_controller() -> Result<()> {
	// TODO: Initialize PIC or APIC
	// For now, just set up basic PIC configuration
	unsafe {
		// Remap PIC interrupts to avoid conflicts with CPU exceptions
		crate::arch::x86_64::pic::init_pic();
	}
	Ok(())
}

/// Initialize exception handlers
fn init_exception_handlers() -> Result<()> {
	// Architecture-specific IDT initialization is done in init()
	Ok(())
}

/// Register an interrupt handler - Linux compatible
pub fn request_irq(
	irq: u32,
	handler: IrqHandler,
	flags: u32,
	name: &'static str,
	dev_id: *mut u8,
) -> Result<()> {
	let mut subsystem = INTERRUPT_SUBSYSTEM.lock();

	if let Some(desc) = subsystem.get_descriptor_mut(irq) {
		let action = IrqAction::new(handler, flags, name, dev_id);

		// Check if IRQ is shared
		if flags & irq_flags::IRQF_SHARED != 0 {
			// Add to action chain
			let mut current = &mut desc.action;
			while let Some(ref mut act) = current {
				current = &mut act.next;
			}
			*current = Some(Box::new(action));
		} else {
			// Replace existing action
			desc.action = Some(Box::new(action));
		}

		// Enable the interrupt
		desc.enable();

		crate::info!("Registered IRQ {} handler: {}", irq, name);
		Ok(())
	} else {
		Err(Error::InvalidArgument)
	}
}

/// Unregister an interrupt handler - Linux compatible
pub fn free_irq(irq: u32, dev_id: *mut u8) -> Result<()> {
	let mut subsystem = INTERRUPT_SUBSYSTEM.lock();

	if let Some(desc) = subsystem.get_descriptor_mut(irq) {
		// Remove action with matching dev_id
		let prev: Option<&mut Box<IrqAction>> = None;
		let current = &mut desc.action;
		let mut found = false;

		// Handle first element specially
		if let Some(ref mut action) = current {
			if action.dev_id.as_ptr() == dev_id {
				*current = action.next.take();
				found = true;
			} else {
				// Search in the chain
				let mut node = current.as_mut().unwrap();
				while let Some(ref mut next_action) = node.next {
					if next_action.dev_id.as_ptr() == dev_id {
						node.next = next_action.next.take();
						found = true;
						break;
					}
					node = node.next.as_mut().unwrap();
				}
			}
		}

		if found {
			// If no more actions, disable the interrupt
			if desc.action.is_none() {
				desc.disable();
			}

			crate::info!("Freed IRQ {} handler", irq);
			Ok(())
		} else {
			Err(Error::NotFound)
		}
	} else {
		Err(Error::InvalidArgument)
	}
}

/// Enable interrupts globally
pub fn enable() {
	#[cfg(target_arch = "x86_64")]
	unsafe {
		core::arch::asm!("sti");
	}
}

/// Disable interrupts globally
pub fn disable() {
	#[cfg(target_arch = "x86_64")]
	unsafe {
		core::arch::asm!("cli");
	}
}

/// Enable a specific interrupt line
pub fn enable_irq(irq: u32) -> Result<()> {
	let mut subsystem = INTERRUPT_SUBSYSTEM.lock();

	if let Some(desc) = subsystem.get_descriptor_mut(irq) {
		desc.enable();
		crate::debug!("Enabled IRQ {}", irq);
		Ok(())
	} else {
		Err(Error::InvalidArgument)
	}
}

/// Disable a specific interrupt line
pub fn disable_irq(irq: u32) -> Result<()> {
	let mut subsystem = INTERRUPT_SUBSYSTEM.lock();

	if let Some(desc) = subsystem.get_descriptor_mut(irq) {
		desc.disable();
		crate::debug!("Disabled IRQ {}", irq);
		Ok(())
	} else {
		Err(Error::InvalidArgument)
	}
}

/// Register an interrupt handler at a specific vector
pub fn register_interrupt_handler(vector: u32, handler: usize) -> Result<()> {
	if vector > 255 {
		return Err(Error::InvalidArgument);
	}

	crate::info!(
		"Registered interrupt handler at vector 0x{:x} -> 0x{:x}",
		vector,
		handler
	);
	Ok(())
}

// Exception handlers
/// System call interrupt handler
#[no_mangle]
pub extern "C" fn syscall_handler() {
	// TODO: Get syscall arguments from registers
	// In x86_64, syscall arguments are passed in:
	// rax = syscall number
	// rdi = arg0, rsi = arg1, rdx = arg2, r10 = arg3, r8 = arg4, r9 = arg5

	let mut syscall_num: u64;
	let mut arg0: u64;
	let mut arg1: u64;
	let mut arg2: u64;
	let mut arg3: u64;
	let mut arg4: u64;
	let mut arg5: u64;

	unsafe {
		core::arch::asm!(
		    "mov {0}, rax",
		    "mov {1}, rdi",
		    "mov {2}, rsi",
		    "mov {3}, rdx",
		    "mov {4}, r10",
		    "mov {5}, r8",
		    "mov {6}, r9",
		    out(reg) syscall_num,
		    out(reg) arg0,
		    out(reg) arg1,
		    out(reg) arg2,
		    out(reg) arg3,
		    out(reg) arg4,
		    out(reg) arg5,
		);
	}

	// Call syscall dispatcher
	let result = crate::syscalls::arch::syscall_entry(
		syscall_num,
		arg0,
		arg1,
		arg2,
		arg3,
		arg4,
		arg5,
	);

	// Return result in register (rax)
	unsafe {
		core::arch::asm!(
		    "mov rax, {0}",
		    in(reg) result,
		);

		// Return from interrupt
		core::arch::asm!("iretq", options(noreturn));
	}
}

/// Install syscall interrupt handler
pub fn install_syscall_handler() -> Result<()> {
	// Install at interrupt vector 0x80 (traditional Linux syscall vector)
	register_interrupt_handler(0x80, syscall_handler as usize)?;

	// TODO: Also set up SYSCALL/SYSRET for x86_64
	Ok(())
}
