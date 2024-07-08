use std::{fmt::Debug, marker::PhantomData};

use holder::{Holdable, Holder};

fn main() {
    let mut wrapper = Wrapper::<usize> {
        value: Token::<i32>(PhantomData),
        _marker: PhantomData,
        game_state: GameState::Idle,
    };
    let token: &mut Token<i32> = wrapper.token_mut();
    let token: &Token<i32> = wrapper.token();
    let game_state: &GameState = wrapper.game_state();
    let game_state_mut: &mut GameState = wrapper.game_state_mut();

    let value = Wrapper2 { value: MyStruct };
    let v: &MyStruct = value.my_struct();
}

#[derive(Holder)]
struct Wrapper<T: Default>
where
    T: Debug,
{
    #[hold]
    value: Token<i32>,
    #[hold(GameStateHolder)]
    game_state: GameState,
    _marker: PhantomData<T>,
}

#[derive(Holdable)]
struct Token<T: Default>(PhantomData<T>)
where
    T: Debug;

#[derive(Holdable)]
enum GameState {
    Idle,
    Play,
}

#[derive(Holder)]
struct Wrapper2<T> {
    #[hold_generic]
    value: T,
}

#[derive(Holdable)]
struct MyStruct;
