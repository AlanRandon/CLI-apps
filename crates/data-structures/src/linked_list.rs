use std::{
    alloc::{AllocError, Allocator, Global, Layout},
    ptr::NonNull,
};

pub struct Node<T> {
    previous: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    data: T,
}

impl<T> Node<T> {
    fn allocate(data: T, allocator: impl Allocator) -> Result<NonNull<Self>, AllocError> {
        let node = Self {
            previous: None,
            next: None,
            data,
        };
        let node_ptr: NonNull<Self> = allocator.allocate(Layout::new::<Self>())?.cast();
        unsafe {
            node_ptr.as_ptr().write(node);
        }
        Ok(node_ptr)
    }
}

pub struct LinkedList<T, A: Allocator = Global> {
    allocator: A,
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

impl<T> LinkedList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::new_in(Global)
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A: Allocator> LinkedList<T, A> {
    fn new_in(allocator: A) -> Self {
        Self {
            allocator,
            head: None,
            tail: None,
            len: 0,
        }
    }

    fn push_head(&mut self, data: T) -> Result<(), AllocError> {
        let node = Node::allocate(data, &self.allocator)?;
        {
            let node = unsafe { &mut *node.as_ptr() };
            node.next = self.head;
        }
        self.head = Some(node);

        if self.len == 0 {
            self.tail = Some(node);
        }

        self.len += 1;

        Ok(())
    }

    fn push_tail(&mut self, data: T) -> Result<(), AllocError> {
        let node = Node::allocate(data, &self.allocator)?;
        {
            let node = unsafe { &mut *node.as_ptr() };
            node.previous = self.tail;
        }
        self.tail = Some(node);

        if self.len == 0 {
            self.head = Some(node);
        }

        self.len += 1;

        Ok(())
    }
}

impl<T, A: Allocator> Drop for LinkedList<T, A> {
    fn drop(&mut self) {
        let mut cursor = Cursor(self.head);
        while let Some(node) = cursor.next_node() {
            unsafe {
                node.as_ptr().drop_in_place();
                self.allocator
                    .deallocate(node.cast(), Layout::new::<Node<T>>());
            }
        }
    }
}

struct Cursor<T>(Option<NonNull<Node<T>>>);

impl<T> Cursor<T> {
    fn next_node(&mut self) -> Option<NonNull<Node<T>>> {
        let node = self.0?;
        self.0 = unsafe { node.as_ref().next };
        Some(node)
    }

    fn previous_node(&mut self) -> Option<NonNull<Node<T>>> {
        let node = self.0?;
        self.0 = unsafe { node.as_ref().previous };
        Some(node)
    }
}

#[test]
fn linked_list() {
    use crate::test::*;

    let mut list = LinkedList::new();
    list.push_head(DropWrapper(1)).unwrap();
}
