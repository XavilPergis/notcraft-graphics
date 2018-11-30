#![feature(trace_macros)]

#[macro_use]
pub mod error;
#[macro_use]
pub mod layout;

pub mod buffer;
pub mod context;
mod draw;
pub mod limits;
pub mod misc;
pub mod program;
pub mod texture;
pub mod texture_array;
pub mod vertex_array;

pub use self::{
    buffer::{Buffer, BufferBuilder, UsageType},
    context::Context,
    draw::{BufferIndex, PrimitiveType},
};

pub struct Cons<T, L>(pub T, pub L);
pub struct Nil;

#[macro_export]
macro_rules! cons {
    ($head:expr, $($tail:tt)*) => {
        Cons($head, cons!($($tail)*))
    };

    ($head:expr) => {
        Cons($head, Nil)
    };
}

#[macro_export]
macro_rules! cons_ty {
    ($head:ty, $($tail:tt)*) => {
        Cons<$head, cons_ty!($($tail)*)>
    };

    ($head:ty) => {
        Cons<$head, Nil>
    };
}
