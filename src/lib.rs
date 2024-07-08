pub use holder_derive::{Holdable, Holder};

pub trait Holder<T> {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait Holding {}
