# `holder` 
getter setter auto gen with holder trait

# example 
```rust
use holder::{Holder, Holdable};

#[derive(Holder)]
struct Wrapper {
    #[hold]
    value: Token
}

#[derive(Holdable)]
struct Token(u32);

#[test]
fn holder_test() {
    let mut wrapper = Wrapper { value: Token(123) };
    let token: &Token = wrapper.token();
    let token_mut: &mut Token = wrapper.token_mut();
}
```
