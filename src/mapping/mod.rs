pub trait Mapper<T> {
    fn map(&self) -> T;
    fn map_to(&self, destination: T) -> T { destination }
}