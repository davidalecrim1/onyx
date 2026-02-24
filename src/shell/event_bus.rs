use std::collections::HashMap;

type HandlerFn = Box<dyn FnMut(&str) + Send>;

pub struct EventBus {
    handlers: HashMap<String, Vec<HandlerFn>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus { handlers: HashMap::new() }
    }

    /// Subscribes a closure to a named event; multiple subscribers are all called in order.
    pub fn subscribe<F>(&mut self, event: &str, f: F)
    where
        F: FnMut(&str) + Send + 'static,
    {
        self.handlers.entry(event.to_string()).or_default().push(Box::new(f));
    }

    /// Emits a named event, calling all subscribers with the given payload string.
    pub fn emit(&mut self, event: &str, payload: &str) {
        if let Some(handlers) = self.handlers.get_mut(event) {
            for handler in handlers.iter_mut() {
                handler(payload);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_receives_event() {
        let mut bus = EventBus::new();
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count2 = count.clone();
        bus.subscribe("buffer.changed", move |_payload| {
            count2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        bus.emit("buffer.changed", "");
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn multiple_subscribers_all_called() {
        let mut bus = EventBus::new();
        let a = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let b = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let a2 = a.clone();
        let b2 = b.clone();
        bus.subscribe("file.opened", move |_| { a2.store(true, std::sync::atomic::Ordering::SeqCst); });
        bus.subscribe("file.opened", move |_| { b2.store(true, std::sync::atomic::Ordering::SeqCst); });
        bus.emit("file.opened", "/path/to/file.md");
        assert!(a.load(std::sync::atomic::Ordering::SeqCst));
        assert!(b.load(std::sync::atomic::Ordering::SeqCst));
    }
}
