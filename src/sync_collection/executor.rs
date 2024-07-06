pub trait Executor<'a> {
    fn submit<F>(&mut self, func: F)
    where
        F: FnOnce() + Send + 'a;

    fn collect(&mut self);
}