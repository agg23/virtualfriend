pub trait TimeSource {
    fn time_ns(&self) -> u64;
}

/// Represents a sink
pub trait Sink<T> {
    /// Push a value out the sink
    fn append(&mut self, value: T);
}

/// Represents a sink of value references.
pub trait SinkRef<T: ?Sized> {
    fn append(&mut self, value: &T);
}

pub type AudioFrame = (i16, i16);
