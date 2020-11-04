use crate::consumer::Consumer;
use crate::producer::Producer;

pub trait IntoIteratorProducer<'a, D, I: Iterator<Item = D>> {
    fn pipe<C: Consumer<'a, D>>(self, consumer: &'a C) -> IteratorProducer<'a, D, I, C>;
}

impl<'a, D, I: IntoIterator<Item = D>> IntoIteratorProducer<'a, D, I::IntoIter> for I {
    fn pipe<C: Consumer<'a, D>>(self, consumer: &'a C) -> IteratorProducer<'a, D, I::IntoIter, C> {
        IteratorProducer {
            iter: self.into_iter(),
            consumer,
        }
    }
}

pub struct IteratorProducer<'a, D, I: Iterator<Item = D>, C: Consumer<'a, D>> {
    iter: I,
    consumer: &'a C,
}

impl<'a, D: Send, I: Iterator<Item = D> + Send, C: Consumer<'a, D>> Producer<'a>
    for IteratorProducer<'a, D, I, C>
{
    type Data = D;
    type Consumer = C;

    fn get_next_product(&mut self) -> Option<Self::Data> {
        self.iter.next()
    }

    fn consumer(&self) -> &'a Self::Consumer {
        self.consumer
    }
}
