use std::{
    alloc::{AllocError, Allocator, Global, Layout},
    fmt::Debug,
    marker::PhantomData,
    ptr::NonNull,
};

pub struct Node<T> {
    previous: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    pub data: T,
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

impl<T, A: Allocator + Default> Default for LinkedList<T, A> {
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<T, A: Allocator> LinkedList<T, A> {
    pub fn new_in(allocator: A) -> Self {
        Self {
            allocator,
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_head(&mut self, data: T) -> Result<(), AllocError> {
        let node = Node::allocate(data, &self.allocator)?;

        if let Some(mut head) = self.head {
            unsafe { head.as_mut().previous = Some(node) };
        }

        {
            let node = unsafe { &mut *node.as_ptr() };
            node.next = self.head;
        }

        self.head = Some(node);

        if self.is_empty() {
            self.tail = Some(node);
        }

        self.len += 1;

        Ok(())
    }

    pub fn push_tail(&mut self, data: T) -> Result<(), AllocError> {
        let node = Node::allocate(data, &self.allocator)?;

        if let Some(mut tail) = self.tail {
            unsafe { tail.as_mut().next = Some(node) };
        }

        {
            let node = unsafe { &mut *node.as_ptr() };
            node.previous = self.tail;
        }

        self.tail = Some(node);

        if self.is_empty() {
            self.head = Some(node);
        }

        self.len += 1;

        Ok(())
    }

    pub fn pop_head(&mut self) -> Option<T> {
        let head_ptr = self.head?;

        let head = unsafe { head_ptr.as_ptr().read() };
        unsafe {
            self.allocator
                .deallocate(head_ptr.cast(), Layout::new::<Node<T>>());
        };

        self.head = head.next;
        self.len -= 1;

        if self.is_empty() {
            self.tail = None;
        }

        Some(head.data)
    }

    pub fn pop_tail(&mut self) -> Option<T> {
        let tail_ptr = self.tail?;

        let tail = unsafe { tail_ptr.as_ptr().read() };
        unsafe {
            self.allocator
                .deallocate(tail_ptr.cast(), Layout::new::<Node<T>>());
        };

        self.tail = tail.previous;
        self.len -= 1;

        if self.is_empty() {
            self.head = None;
        }

        Some(tail.data)
    }

    pub fn clear(&mut self) {
        let mut node = self.head;
        loop {
            let Some(node_ptr) = node else {
                break;
            };

            node = unsafe { node_ptr.as_ref().next };

            unsafe {
                node_ptr.as_ptr().drop_in_place();
                self.allocator
                    .deallocate(node_ptr.cast(), Layout::new::<Node<T>>());
            }
        }
        self.head = None;
        self.tail = None;
        self.len = 0;
    }
}

impl<T, A: Allocator> Drop for LinkedList<T, A> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, A: Allocator> LinkedList<T, A> {
    pub fn cursor_head(&self) -> Cursor<T> {
        Cursor {
            node: self.head,
            _phantom: PhantomData,
        }
    }

    pub fn cursor_tail(&self) -> Cursor<T> {
        Cursor {
            node: self.tail,
            _phantom: PhantomData,
        }
    }

    pub fn cursor_head_mut(&mut self) -> CursorMut<T> {
        CursorMut {
            node: self.head,
            _phantom: PhantomData,
        }
    }

    pub fn cursor_tail_mut(&mut self) -> CursorMut<T> {
        CursorMut {
            node: self.tail,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            node: self.head,
            _phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            node: self.head,
            _phantom: PhantomData,
        }
    }
}

impl<T: Debug, A: Allocator> Debug for LinkedList<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, A: Allocator + Default> FromIterator<T> for LinkedList<T, A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new_in(A::default());
        for item in iter {
            list.push_tail(item).expect("failed to allocate node");
        }
        list
    }
}

pub struct Cursor<'a, T> {
    node: Option<NonNull<Node<T>>>,
    _phantom: PhantomData<&'a LinkedList<T>>,
}

impl<'a, T> Cursor<'a, T> {
    pub fn next_node(&mut self) -> Option<&Node<T>> {
        let node = unsafe { self.node?.as_ref() };
        self.node = node.next;
        Some(node)
    }

    pub fn previous_node(&mut self) -> Option<&Node<T>> {
        let node = unsafe { self.node?.as_ref() };
        self.node = node.previous;
        Some(node)
    }

    #[must_use]
    pub fn current(&self) -> Option<&Node<T>> {
        let node = unsafe { self.node?.as_ref() };
        Some(node)
    }
}

pub struct CursorMut<'a, T> {
    node: Option<NonNull<Node<T>>>,
    _phantom: PhantomData<&'a mut LinkedList<T>>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn next_node(&mut self) -> Option<&mut Node<T>> {
        let node = unsafe { self.node?.as_mut() };
        self.node = node.next;
        Some(node)
    }

    pub fn previous_node(&mut self) -> Option<&mut Node<T>> {
        let node = unsafe { self.node?.as_mut() };
        self.node = node.previous;
        Some(node)
    }

    #[must_use]
    pub fn current(&self) -> Option<&Node<T>> {
        let node = unsafe { self.node?.as_ref() };
        Some(node)
    }
}

pub struct Iter<'a, T> {
    node: Option<NonNull<Node<T>>>,
    _phantom: PhantomData<&'a LinkedList<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.node?.as_ref() };
        self.node = node.next;
        Some(&node.data)
    }
}

impl<'a, T, A: Allocator> IntoIterator for &'a LinkedList<T, A> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, T> {
    node: Option<NonNull<Node<T>>>,
    _phantom: PhantomData<&'a mut LinkedList<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.node?.as_mut() };
        self.node = node.next;
        Some(&mut node.data)
    }
}

impl<'a, T, A: Allocator> IntoIterator for &'a mut LinkedList<T, A> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<T, A: Allocator> {
    node: Option<NonNull<Node<T>>>,
    list: LinkedList<T, A>,
}

impl<T: Debug, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let node_ptr = self.node?;
        let node = unsafe { node_ptr.as_ptr().read() };
        self.node = node.next;
        self.list.head = node.next;
        unsafe {
            self.list
                .allocator
                .deallocate(node_ptr.cast(), Layout::new::<Node<T>>());
        }
        Some(node.data)
    }
}

impl<T: Debug, A> IntoIterator for LinkedList<T, A>
where
    A: Allocator + ToOwned<Owned = A>,
{
    type Item = T;
    type IntoIter = IntoIter<T, A>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            node: self.head,
            list: self,
        }
    }
}

#[test]
fn linked_list() {
    use crate::test::*;

    let counter = AllocationCounter::new();

    let mut list = [1, 2, 3, 4]
        .map(|i| Arc::clone(&counter).count(i))
        .into_iter()
        .collect::<LinkedList<_, TestingAllocator>>();

    for i in &mut list {
        i.data *= 2;
    }

    assert_eq!(
        vec![2, 4, 6, 8],
        list.into_iter().map(|i| i.data).collect::<Vec<_>>()
    );
}
