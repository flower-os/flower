//! # PS/2 Driver
//!
//! The PS/2 driver provides interface into the PS/2 controller, allowing access to devices using this protocol.
//! The controller is accessed through the static `CONTROLLER` field.
//!
//! The [Controller] handles all interface with the controller for devices.
//! For it to be initialized, `initialize` must be called on it. This sets up all attached devices.
//!
//! The [Device] handles interface to a single PS/2 device. Its state can be checked and toggled through `enable` and `disable`.
//! Devices can be obtained from the controller through `device(DevicePort)` or `devices`.

pub mod io;

use drivers::ps2::io::Ps2Error;
use drivers::ps2::io::commands::{self, ControllerCommand, ControllerReturnCommand, ControllerDataCommand, DeviceCommand, DeviceDataCommand};
use spin::Mutex;

pub const RESEND: u8 = 0xFE;
pub const ACK: u8 = 0xFA;

lazy_static! {
    pub static ref CONTROLLER: Mutex<Controller> = Mutex::new(Controller::new());
}

bitflags! {
    pub struct ConfigFlags: u8 {
        /// If interrupts for Port 1 are enabled
        const PORT_INTERRUPT_1 = 1 << 0;
        /// If interrupts for Port 2 are enabled
        const PORT_INTERRUPT_2 = 1 << 1;
        /// If the clock for Port 1 is disabled
        const PORT_CLOCK_1 = 1 << 4;
        /// If the clock for Port 2 is disabled
        const PORT_CLOCK_2 = 1 << 5;
        /// If the controller will transform scan set 2 to scan set 1
        const PORT_TRANSLATION_1 = 1 << 6;
    }
}

/// Represents the state of a device.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DeviceState {
    /// The device does not exist or hasn't been detected yet
    Unavailable,
    /// The device has been detected but is not enabled
    Available,
    /// The device has been enabled
    Enabled,
}

/// Represents the port of a device
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DevicePort {
    /// The device is in the keyboard port
    Keyboard,
    /// The device is in the mouse port
    Mouse,
}

/// Represents the PS2 master controller
pub struct Controller {
    pub devices: (Device, Device),
    pub config: ConfigFlags,
}

impl Controller {
    fn new() -> Self {
        Controller {
            devices: (
                Device::new(DevicePort::Keyboard),
                Device::new(DevicePort::Mouse),
            ),
            config: ConfigFlags::empty(),
        }
    }

    /// Initializes this PS2 controller
    pub fn initialize(&mut self) -> Result<(), Ps2Error> {
        info!("ps2c: initializing");

        self.prepare_devices()?;
        debug!("ps2c: disabled devices");

        io::flush_output()?;

        self.initialize_config()?;

        if !self.test_controller()? {
            error!("ps2c: controller test failed");
        }

        debug!("ps2c: testing devices");
        match self.test_devices()? {
            (false, _) => warn!("ps2c: first device not supported"),
            (_, false) => warn!("ps2c: second device not supported"),
            _ => (),
        }

        // Check if any devices are available
        if self.reset_devices()? > 0 {
            debug!("ps2c: prepared devices");
        } else {
            info!("ps2c: detected no available devices");
        }

        io::flush_output()?;

        Ok(())
    }

    /// Writes the given config to the PS2 controller
    pub fn write_config(&self, config: ConfigFlags) -> Result<(), Ps2Error> {
        commands::send_data(ControllerDataCommand::WriteConfig, config.bits())
    }

    /// Reads the config from the PS2 controller
    pub fn read_config(&self) -> Result<ConfigFlags, Ps2Error> {
        let read = commands::send_ret(ControllerReturnCommand::ReadConfig)?;

        Ok(ConfigFlags::from_bits_truncate(read))
    }

    /// Gets the device for the given port
    #[allow(dead_code)] // To be used by drivers interfacing with PS/2
    pub fn device(&mut self, port: DevicePort) -> &mut Device {
        if port == DevicePort::Keyboard {
            &mut self.devices.0
        } else {
            &mut self.devices.1
        }
    }

    /// Resets this controller's devices and prepares them for initialization
    fn prepare_devices(&mut self) -> Result<(), Ps2Error> {
        self.devices.0.disable()?;
        self.devices.1.disable()?;

        self.devices.0.state = DeviceState::Unavailable;
        self.devices.1.state = DeviceState::Unavailable;

        Ok(())
    }

    /// Initializes the config for this controller
    fn initialize_config(&mut self) -> Result<(), Ps2Error> {
        // Read the config from the controller
        self.config = self.read_config()?;

        // Set all required config flags
        self.config.set(ConfigFlags::PORT_INTERRUPT_1, false);
        self.config.set(ConfigFlags::PORT_INTERRUPT_2, false);
        self.config.set(ConfigFlags::PORT_TRANSLATION_1, false);

        // Write the updated config back to the controller
        self.write_config(self.config)?;

        debug!("ps2c: initialized config");

        Ok(())
    }

    /// Tests this controller
    fn test_controller(&self) -> Result<bool, Ps2Error> {
        Ok(commands::send_ret(ControllerReturnCommand::TestController)? == 0x55)
    }

    /// Tests all of this controller's devices
    fn test_devices(&mut self) -> Result<(bool, bool), Ps2Error> {
        // Check if controller supports the second device
        if self.config.contains(ConfigFlags::PORT_CLOCK_2) {
            self.devices.1.enable()?;
            self.config = self.read_config()?;
            self.devices.1.disable()?;
        }

        // Test both devices
        let first_supported = self.devices.0.test()?;
        let second_supported = !self.config.contains(ConfigFlags::PORT_CLOCK_2) && self.devices.1.test()?;

        Ok((first_supported, second_supported))
    }

    /// Resets all devices and returns the count available
    fn reset_devices(&mut self) -> Result<u8, Ps2Error> {
        let mut available_count = 0;

        if self.devices.0.state == DeviceState::Available {
            self.devices.0.reset()?;
            available_count += 1;
        }

        if self.devices.1.state == DeviceState::Available {
            self.devices.1.reset()?;
            available_count += 1;
        }

        Ok(available_count)
    }
}

/// Represents a PS2 device
pub struct Device {
    pub state: DeviceState,
    pub port: DevicePort,
}

impl Device {
    const fn new(port: DevicePort) -> Self {
        Device {
            state: DeviceState::Unavailable,
            port,
        }
    }

    /// Tests this device to see if it is available
    pub fn test(&mut self) -> Result<bool, Ps2Error> {
        let cmd = if self.port == DevicePort::Mouse {
            ControllerReturnCommand::TestPort2
        } else {
            ControllerReturnCommand::TestPort1
        };
        let available = commands::send_ret(cmd)? == 0x0;

        if available {
            self.state = DeviceState::Available;
        } else {
            self.state = DeviceState::Unavailable;
        }

        Ok(available)
    }

    /// Enables this device
    pub fn enable(&mut self) -> Result<(), Ps2Error> {
        let cmd = if self.port == DevicePort::Mouse {
            ControllerCommand::EnablePort2
        } else {
            ControllerCommand::EnablePort1
        };
        commands::send(cmd)?;
        self.state = DeviceState::Enabled;

        Ok(())
    }

    /// Disables this device
    pub fn disable(&mut self) -> Result<(), Ps2Error> {
        let cmd = if self.port == DevicePort::Mouse {
            ControllerCommand::DisablePort2
        } else {
            ControllerCommand::DisablePort1
        };
        commands::send(cmd)?;
        self.state = DeviceState::Available;

        Ok(())
    }

    /// Resets this device
    pub fn reset(&mut self) -> Result<(), Ps2Error> {
        self.command(DeviceCommand::Reset)?;

        Ok(())
    }

    /// Sends a command for this PS2 device and returns result
    pub fn command(&mut self, cmd: DeviceCommand) -> Result<u8, Ps2Error> {
        self.command_raw(cmd as u8)
    }

    /// Sends a command for this PS2 device with data and returns a result
    #[allow(dead_code)] // To be used by drivers interfacing with PS/2
    pub fn command_data(&mut self, cmd: DeviceDataCommand, data: u8) -> Result<u8, Ps2Error> {
        if self.state != DeviceState::Unavailable {
            self.command_raw(cmd as u8).and_then(|result| match result {
                ACK => {
                    io::DATA_PORT.with_lock(|mut data_port| {
                        io::write(&mut data_port, data as u8)?;
                        io::read(&mut data_port)
                    })
                }
                _ => Ok(result)
            })
        } else {
            Err(Ps2Error::DeviceUnavailable)
        }
    }

    /// Sends a raw command code to this device
    fn command_raw(&mut self, cmd: u8) -> Result<u8, Ps2Error> {
        if self.state != DeviceState::Unavailable {
            // If second PS2 port, send context switch command
            if self.port == DevicePort::Mouse {
                commands::send(ControllerCommand::WriteInputPort2)?;
            }

            io::DATA_PORT.with_lock(|mut data_port| {
                for _ in 0..4 {
                    io::write(&mut data_port, cmd)?;
                    match io::read(&mut data_port) {
                        Ok(RESEND) => continue,
                        result => return result,
                    }
                }

                Err(Ps2Error::NoData)
            })
        } else {
            Err(Ps2Error::DeviceUnavailable)
        }
    }
}
