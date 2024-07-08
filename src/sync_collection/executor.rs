pub trait Executor<'a> {
    fn submit(&mut self, func: fn());

    fn collect(&mut self);
}