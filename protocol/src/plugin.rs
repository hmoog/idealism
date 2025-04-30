pub trait Plugin {
    fn start(&self) {}

    fn shutdown(&self) {}
}
