use core::sync::atomic::Ordering;

pub fn print(message: &str) {
    tracing::info!(context=crate::context::CURRENT_CONTEXT.load(Ordering::Relaxed), "print {}", message);
}
