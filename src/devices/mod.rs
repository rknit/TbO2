mod serial;

pub use serial::SerialIO;

#[allow(unused_variables)]
pub trait Device {
    fn attach(&mut self) {}

    fn detach(&mut self) {}

    fn reset(&mut self) {}

    #[must_use]
    fn read(&mut self, addr: usize) -> Option<u8> {
        None
    }

    fn write(&mut self, addr: usize, data: u8) -> Option<()> {
        None
    }
}
