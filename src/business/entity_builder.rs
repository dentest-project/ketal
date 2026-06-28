pub trait EntityBuilder<E>: Sized {
    fn init() -> Self;

    fn build(self) -> E;
}
