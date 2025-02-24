#[allow(unused_variables)]
pub trait Device {
    fn on_attach(&mut self) {}

    fn on_detach(&mut self) {}

    fn on_reset(&mut self) {}

    #[must_use]
    fn on_read(&self, addr: usize) -> Option<u8> {
        None
    }

    fn on_write(&mut self, addr: usize, data: u8) -> Option<()> {
        None
    }
}
