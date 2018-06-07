//! Extension trait and implementation for observing and action on streamed data.

use Data;
use dataflow::channels::pact::Pipeline;
use dataflow::{Stream, Scope};
use dataflow::operators::generic::unary::Unary;

/// Methods to inspect records and batches of records on a stream.
pub trait Inspect<G: Scope, D: Data>: Sized {
    /// Runs a supplied closure on each observed data element.
    ///
    /// #Examples
    /// ```
    /// use timely::dataflow::operators::{ToStream, Map, Inspect};
    ///
    /// timely::example(|scope| {
    ///     (0..10).to_stream(scope)
    ///            .inspect(|x| println!("seen: {:?}", x));
    /// });
    /// ```
    fn inspect(&self, mut func: impl FnMut(&D)+'static) -> Self {
        self.inspect_batch(move |_, data| {
            for datum in data.iter() { func(datum); }
        })
    }

    /// Runs a supplied closure on each observed data element and associated time.
    ///
    /// #Examples
    /// ```
    /// use timely::dataflow::operators::{ToStream, Map, Inspect};
    ///
    /// timely::example(|scope| {
    ///     (0..10).to_stream(scope)
    ///            .inspect_time(|t, x| println!("seen at: {:?}\t{:?}", t, x));
    /// });
    /// ```
    fn inspect_time(&self, mut func: impl FnMut(&G::Timestamp, &D)+'static) -> Self {
        self.inspect_batch(move |time, data| {
            for datum in data.iter() {
                func(&time, &datum);
            }
        })
    }

    /// Runs a supplied closure on each observed data batch (time and data slice).
    ///
    /// #Examples
    /// ```
    /// use timely::dataflow::operators::{ToStream, Map, Inspect};
    ///
    /// timely::example(|scope| {
    ///     (0..10).to_stream(scope)
    ///            .inspect_batch(|t,xs| println!("seen at: {:?}\t{:?} records", t, xs.len()));
    /// });
    /// ```
    fn inspect_batch(&self, func: impl FnMut(&G::Timestamp, &[D])+'static) -> Self;
}

impl<G: Scope, D: Data> Inspect<G, D> for Stream<G, D> {
    fn inspect_batch(&self, mut func: impl FnMut(&G::Timestamp, &[D])+'static) -> Stream<G, D> {
        self.unary_stream(Pipeline, "InspectBatch", move |input, output| {
            input.for_each(|time, data| {
                func(&time, &data[..]);
                output.session(&time).give_content(data);
            });
        })
    }
}
