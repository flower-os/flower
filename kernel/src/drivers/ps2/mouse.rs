use crate::drivers::ps2;

pub struct Mouse;

impl ps2::io::CommandIo for Mouse {
    #[inline]
    fn send(command: u8) -> ps2::Result<()> {
        ps2::device::send_raw_device_command(command, true)
    }

    #[inline]
    fn send_data(command: u8, data: u8) -> ps2::Result<()> {
        ps2::device::send_raw_device_command_data(command, data, true)
    }

    fn read() -> ps2::Result<u8> {
        ps2::io::read(&ps2::io::DATA_PORT).ok_or(ps2::Error::ExpectedResponse)
    }
}

impl ps2::Device for Mouse {
    #[inline]
    fn enable() -> ps2::Result<()> { ps2::Controller::enable_mouse() }

    #[inline]
    fn disable() -> ps2::Result<()> { ps2::Controller::disable_mouse() }

    #[inline]
    fn test() -> ps2::Result<bool> { ps2::Controller::test_mouse() }
}
