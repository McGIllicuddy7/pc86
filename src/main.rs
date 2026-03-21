pub mod input;
pub mod io;
pub mod managed;
pub use managed::*;
make_ecs! {
TEST_ECS, (
    INT_BUFFER, IntegerComp, i32, add_integer, remove_integer, get_integer, get_integer_mut, get_integer_ref
    )
}
pub fn main() {}

