use gl;

use crate::buffer::{Buffer, BufferTarget};
use crate::layout::Layout;
use crate::limits::Limits;
use crate::program::program::Program;
use crate::vertex_array::VertexArray;
use crate::BufferIndex;
use crate::PrimitiveType;

use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
    sync::Mutex,
};

pub(crate) struct Entry<'v, V>(
    ::std::collections::hash_map::Entry<'v, TypeId, Box<dyn Any>>,
    PhantomData<*const V>,
);

impl<'v, V: 'static> Entry<'v, V> {
    fn or_insert_with<F: FnOnce() -> VertexArray<V>>(self, func: F) -> &'v mut VertexArray<V> {
        self.0
            .or_insert_with(|| Box::new(func()))
            .downcast_mut()
            .unwrap()
    }
}

#[derive(Debug)]
pub(crate) struct VaoMap {
    table: HashMap<TypeId, Box<dyn Any>>,
}

impl VaoMap {
    pub(crate) fn new() -> Self {
        VaoMap {
            table: HashMap::new(),
        }
    }

    pub(crate) fn entry<'v, V: Any>(&'v mut self) -> Entry<'v, V> {
        Entry(self.table.entry(TypeId::of::<V>()), PhantomData)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ViewportRect {
    pub x: i32,
    pub width: u32,
    pub y: i32,
    pub height: u32,
}

impl ViewportRect {
    fn query() -> Self {
        let mut r = [0i32; 4];
        gl_call!(assert GetIntegerv(gl::VIEWPORT, r.as_mut_ptr()));
        ViewportRect {
            x: r[0],
            y: r[1],
            width: r[2] as u32,
            height: r[3] as u32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context(Rc<ContextInner>);

#[derive(Debug)]
pub struct ContextInner {
    // Make sure this isn't Send or Sync
    _marker: ::std::marker::PhantomData<*mut ()>,

    pub(crate) limits: Limits,
    pub(crate) viewport: Cell<ViewportRect>,

    pub(crate) format_cache: RefCell<VaoMap>,
}

impl Context {
    pub fn load<F>(mut load_fn: F) -> Context
    where
        F: FnMut(&'static str) -> *const (),
    {
        gl::load_with(|symbol| load_fn(symbol) as *const _);

        Context(Rc::new(ContextInner {
            _marker: ::std::marker::PhantomData,

            limits: Limits::load(),
            viewport: Cell::new(ViewportRect::query()),
            format_cache: RefCell::new(VaoMap::new()),
        }))
    }

    // self.ctx.draw_elements(gl::TRIANGLES, &self.vertices, &self.indices);
    pub fn draw_elements<V: Layout, I: BufferIndex>(
        &mut self,
        primitive: PrimitiveType,
        program: &Program,
        vertices: &Buffer<V>,
        indices: &Buffer<I>,
    ) {
        if vertices.len() > 0 {
            program.bind(self);

            // Find a VAO that describes our vertex format, creating one if it does not
            // exist.
            let mut map = self.0.format_cache.borrow_mut();
            let vao = map.entry::<V>().or_insert_with(|| {
                VertexArray::<V>::for_vertex_type(self).with_buffer(self, vertices)
            });

            // set the buffer binding the the buffer that was passed in
            vao.set_buffer(self, vertices);
            vao.bind(self);
            indices.bind(self, BufferTarget::Element);

            gl_call!(assert DrawElements(
                primitive as u32,
                indices.len() as i32,
                I::INDEX_TYPE,
                0 as *const _
            ));
        }
    }

    // self.ctx.draw_elements(gl::TRIANGLES, &shader, &self.vertices);
    pub fn draw_arrays<V: Layout>(
        &mut self,
        primitive: PrimitiveType,
        program: &Program,
        vertices: &Buffer<V>,
    ) {
        if vertices.len() > 0 {
            program.bind(self);

            // Find a VAO that describes our vertex format, creating one if it does not
            // exist.
            let mut map = self.0.format_cache.borrow_mut();
            let vao = map.entry::<V>().or_insert_with(|| {
                VertexArray::<V>::for_vertex_type(self).with_buffer(self, vertices)
            });

            // set the buffer binding the the buffer that was passed in
            vao.set_buffer(self, vertices);
            vao.bind(self);

            gl_call!(assert DrawArrays(
                primitive as u32,
                0,
                vertices.len() as i32
            ));
        }
    }

    pub fn limits(&self) -> &Limits {
        &self.0.limits
    }

    pub fn set_viewport<R>(&self, rect: R)
    where
        R: Into<ViewportRect>,
    {
        let rect = rect.into();
        self.0.viewport.set(rect);
        gl_call!(assert Viewport(rect.x, rect.y, rect.width as i32, rect.height as i32))
    }

    pub fn viewport(&self) -> ViewportRect {
        self.0.viewport.get()
    }
}
