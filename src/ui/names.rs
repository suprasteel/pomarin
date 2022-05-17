pub trait NamedHandle<H> {
    fn name(&self) -> H;
}
