use drivers::ps2::io::*;
use spin::Mutex;

pub const DEVICE_ENABLED_FLAG: u8 = 1 << 0;
pub const DEVICE_SECOND_FLAG: u8 = 1 << 1;
pub const DEVICE_AVAILABLE_FLAG: u8 = 1 << 2;

pub const RESEND: u8 = 0xFE;
pub const ACK: u8 = 0xFA;
pub const SELF_TEST_PASSED: u8 = 0xAA;

pub static PS2: Mutex<Ps2Controller> = Mutex::new(Ps2Controller::new());

/// Represents the PS2 master controller
pub struct Ps2Controller {
    pub initialized: bool,
    pub devices: [Ps2Device; 2],
    config: Ps2Config,
}

impl Ps2Controller {
    const fn new() -> Self {
        Ps2Controller {
            initialized: false,
            devices: [
                Ps2Device {
                    flags: 0,
                },
                Ps2Device {
                    flags: DEVICE_SECOND_FLAG,
                }],
            config: Ps2Config {
                data: 0,
            },
        }
    }

    /// Initializes this PS2 controller
    pub fn initialize(&mut self) {
        println!("ps2c: initializing");

        for device in self.devices.iter_mut() {
            device.disable();
            device.set_flag(DEVICE_AVAILABLE_FLAG, false);
        }

        println!("ps2c: disabled devices");

        flush_output();

        self.initialize_config();

        if !self.test_controller() {
            println!("ps2c: controller test failed");
        }

        // Test the first device
        self.devices[0].test();

        // Check if controller supports the second device
        if !self.config.get(ControllerConfigBit::PortOutputFull2) {
            self.devices[1].enable();
            self.read_config();
            self.devices[1].disable();
        }

        // Test the second device
        if self.config.get(ControllerConfigBit::PortOutputFull2) {
            self.devices[1].test();
        } else {
            println!("ps2c: second device unsupported");
        }

        let mut available_count: u8 = 0;
        for device in self.devices.iter_mut() {
            // Enable if device available
            if device.get_flag(DEVICE_AVAILABLE_FLAG) {
                device.enable();
                device.reset();
                available_count += 1;
            }
        }

        // Check if any devices are available
        if available_count > 0 {
            println!("ps2c: enabled devices");
        } else {
            println!("ps2c: detected no available devices");
        }

        flush_output();

        self.initialized = true;
    }

    /// Initializes the config for this controller
    fn initialize_config(&mut self) {
        // Read the config from the controller
        self.read_config();

        // Set all required config flags
        self.config.set(ControllerConfigBit::PortInterrupt1, false);
        self.config.set(ControllerConfigBit::PortInterrupt2, false);
        self.config.set(ControllerConfigBit::PortTranslation1, false);

        // Write the updated config back to the controller
        self.write_config();

        println!("ps2c: initialized config");
    }

    /// Tests this controller
    fn test_controller(&mut self) -> bool {
        command_ret(ControllerReturnCommand::TestController) == Some(0x55)
    }

    /// Writes the current config to the PS2 controller
    pub fn write_config(&mut self) {
        command_data(ControllerCommand::WriteConfig, self.config.data);
    }

    /// Reads the config from the PS2 controller
    pub fn read_config(&mut self) {
        self.config.data = command_ret(ControllerReturnCommand::ReadConfig).unwrap_or(self.config.data);
    }
}

/// Returns if the given code is OK
#[allow(dead_code)] // To be used by keyboard / mouse driver depending on PS/2
pub fn is_ok(code: Option<u8>) -> bool {
    code == Some(ACK) || code == Some(SELF_TEST_PASSED)
}

/// Represents a PS2 device
pub struct Ps2Device {
    flags: u8,
}

impl Ps2Device {
    /// Tests this device to see if it is available
    pub fn test(&mut self) -> bool {
        let available = command_ret(if self.get_flag(DEVICE_SECOND_FLAG) {
            ControllerReturnCommand::TestPort2
        } else {
            ControllerReturnCommand::TestPort1
        }) == Some(0x0);
        self.set_flag(DEVICE_AVAILABLE_FLAG, available);
        available
    }

    /// Enables this device
    pub fn enable(&mut self) {
        if self.get_flag(DEVICE_AVAILABLE_FLAG) {
            command(if self.get_flag(DEVICE_SECOND_FLAG) {
                ControllerCommand::EnablePort2
            } else {
                ControllerCommand::EnablePort1
            });
            self.set_flag(DEVICE_ENABLED_FLAG, true);
        }
    }

    /// Disables this device
    pub fn disable(&mut self) {
        command(if self.get_flag(DEVICE_SECOND_FLAG) {
            ControllerCommand::DisablePort2
        } else {
            ControllerCommand::DisablePort1
        });
        self.set_flag(DEVICE_ENABLED_FLAG, false);
    }

    /// Resets this device
    pub fn reset(&mut self) {
        self.command(DeviceCommand::Reset);
    }

    /// Sends a command for this PS2 device and returns result
    #[allow(dead_code)] // Used by drivers depending on PS/2
    pub fn command(&mut self, cmd: DeviceCommand) -> Option<u8> {
        // If second PS2 port, send context switch command
        if self.get_flag(DEVICE_SECOND_FLAG) {
            command(ControllerCommand::WriteInputPort2);
        }
        let mut result = Some(RESEND);
        for _i in 0..4 {
            if result != Some(RESEND) && result != None {
                break;
            }
            write_data(cmd as u8);
            result = read_data();
        }
        result
    }

    /// Sends a command for this PS2 device with data and returns a result
    #[allow(dead_code)] // Used by drivers depending on PS/2
    pub fn command_data(&mut self, cmd: DeviceCommand, data: u8) -> Option<u8> {
        let result = self.command(cmd);
        if is_ok(result) {
            write_data(data as u8);
            read_data()
        } else {
            result
        }
    }

    /// Sets the given flag bit
    pub fn set_flag(&mut self, bit: u8, value: bool) {
        if value {
            self.flags |= bit as u8;
        } else {
            self.flags &= !(bit as u8);
        }
    }

    /// Gets the given flag bit
    pub fn get_flag(&mut self, bit: u8) -> bool {
        (self.flags & bit) != 0
    }
}

/// Represents the PS2 controller configuration
pub struct Ps2Config {
    data: u8,
}

impl Ps2Config {
    /// Sets the given config bit
    pub fn set(&mut self, bit: ControllerConfigBit, value: bool) {
        if value {
            self.data |= bit as u8;
        } else {
            self.data &= !(bit as u8);
        }
    }

    /// Gets the requested config bit
    pub fn get(&mut self, bit: ControllerConfigBit) -> bool {
        self.data & (bit as u8) != 0
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerConfigBit {
    PortInterrupt1 = 1 << 0,
    PortInterrupt2 = 1 << 1,
    System = 1 << 2,
    KeyboardLock = 1 << 4,
    PortOutputFull2 = 1 << 5,
    PortTranslation1 = 1 << 6,
}
