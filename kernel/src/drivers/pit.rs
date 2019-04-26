/// Programmable Interval Timer controller for sleeping and measuring time passage
/// Note: This is a very low accuracy driver, with a drift of ~1ms every 6 seconds

use crate::interrupts;
use spin::Mutex;
use crate::io::SynchronizedPort;
use core::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub static CONTROLLER: Mutex<Controller> = unsafe { Mutex::new(Controller::new()) };

const BASE_FREQUENCY_HZ: usize = 1193182;
const FREQUENCY_HZ: usize = 1000;

const RELOAD_VALUE: u16 = compute_reload(FREQUENCY_HZ);

const fn compute_reload(frequency: usize) -> u16 {
    (BASE_FREQUENCY_HZ / frequency) as u16
}

fn tick() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}

pub fn sleep(ms: usize) {
    let wake_time = COUNTER.load(Ordering::SeqCst) + ms;
    while COUNTER.load(Ordering::SeqCst) < wake_time {
        // TODO spin for now -- userspace cannot hlt
//        unsafe { asm!("hlt"); }
    }
}

pub fn time_ms() -> usize {
    COUNTER.load(Ordering::SeqCst)
}

pub struct Controller {
    configure_port: SynchronizedPort<u8>,
    channel_0: Channel,
}

impl Controller {
    const unsafe fn new() -> Controller {
        Controller {
            configure_port: SynchronizedPort::new(0x43),
            channel_0: Channel::new(SynchronizedPort::new(0x40)),
        }
    }

    pub fn initialize(&mut self) {
        info!("pit: initializing");

        self.configure(0, OperatingMode::RateGenerator, AccessMode::LobyteHibyte);

        COUNTER.store(0, Ordering::Relaxed);

        interrupts::listen(interrupts::Irq::Pit, tick);

        self.channel_0.set_reload_value(RELOAD_VALUE);
    }

    fn configure(&mut self, channel: u8, operating_mode: OperatingMode, access_mode: AccessMode) {
        let configuration = (channel << 6) | ((access_mode as u8) << 4) | ((operating_mode as u8) << 1);
        self.configure_port.write(configuration);
    }
}

pub struct Channel {
    port: SynchronizedPort<u8>,
}

impl Channel {
    const fn new(port: SynchronizedPort<u8>) -> Channel {
        Channel { port }
    }

    fn set_reload_value(&mut self, reload_value: u16) {
        let lower = (reload_value & 0xFF) as u8;
        let upper = ((reload_value >> 8) & 0xFF) as u8;

        self.port.write(lower);
        self.port.write(upper);
    }
}

/// From: [OsDev Wiki](https://wiki.osdev.org/Programmable_Interval_Timer)
#[repr(u8)]
pub enum OperatingMode {
    InterruptOnTerminalCount = 0,
    HardwareReTriggerableOneShot = 1,
    RateGenerator = 2,
    SquareWaveGenerator = 3,
    SoftwareTriggeredStrobe = 4,
    HardwareTriggeredStrobe = 5,
}

#[repr(u8)]
pub enum AccessMode {
    LobyteOnly = 1,
    HibyteOnly = 2,
    LobyteHibyte = 3,
}
