use std::{fmt::Debug, marker::PhantomData};

use holder::{Holdable, Holder};

fn main() {
    let mut wrapper = Wrapper::<usize> {
        value: Token::<i32>(PhantomData),
        _marker: PhantomData,
    };
    let token: &mut Token<i32> = wrapper.token_mut();
    let token: &Token<i32> = wrapper.token();
}

#[derive(Holder)]
struct Wrapper<T: Default>
where
    T: Debug,
{
    #[hold]
    value: Token<i32>,
    _marker: PhantomData<T>,
}

#[derive(Holdable)]
struct Token<T: Default>(PhantomData<T>)
where
    T: Debug;
