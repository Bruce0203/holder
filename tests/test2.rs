use holder::Holder;

enum GameState {
    Idle,
    Play,
}
trait GameStateHolder {
    fn game_state(&self) -> &GameState;
    fn game_state_mut(&mut self) -> &mut GameState;
}
struct Wrapper2<T, B> {
    //#[hold_generic]
    value: T,
    //#[hold]
    value2: GameState,
    value3: B,
}

fn asdf() {
    let wrapper2 = Wrapper2 {
        value: MyStruct,
        value2: GameState::Idle,
        value3: MyStruct2,
    };
    let game_state: &GameState = wrapper2.game_state();
    let my_struct: &MyStruct = wrapper2.my_struct();
    let my_struct2: &MyStruct2 = wrapper2.my_struct2();
}

struct MyStruct;
struct MyStruct2;

impl<T, B> Holder<1, T> for Wrapper2<T, B> {
    fn get(&self) -> &T {
        &self.value
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, B> Holder<2, B> for Wrapper2<T, B> {
    fn get(&self) -> &B {
        &self.value3
    }

    fn get_mut(&mut self) -> &mut B {
        &mut self.value3
    }
}

impl<'a, T, const N: usize> MyStructHolder<N> for T
where
    T: Holder<N, MyStruct>,
{
    fn my_struct(&self) -> &MyStruct {
        self.get()
    }

    fn my_struct_mut(&mut self) -> &mut MyStruct {
        self.get_mut()
    }
}

impl<'a, T, const N: usize> MyStruct2Holder<N> for T
where
    T: Holder<N, MyStruct2>,
{
    fn my_struct2(&self) -> &MyStruct2 {
        self.get()
    }

    fn my_struct2_mut(&mut self) -> &mut MyStruct2 {
        self.get_mut()
    }
}

impl<T, B> GameStateHolder for Wrapper2<T, B> {
    fn game_state(&self) -> &GameState {
        &self.value2
    }

    fn game_state_mut(&mut self) -> &mut GameState {
        &mut self.value2
    }
}

trait MyStructHolder<const N: usize> {
    fn my_struct(&self) -> &MyStruct;
    fn my_struct_mut(&mut self) -> &mut MyStruct;
}

trait MyStruct2Holder<const N: usize> {
    fn my_struct2(&self) -> &MyStruct2;
    fn my_struct2_mut(&mut self) -> &mut MyStruct2;
}
