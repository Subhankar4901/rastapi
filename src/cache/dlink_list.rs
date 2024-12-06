use std::ptr::NonNull;
pub(crate) struct Node<T> {
    pub(super) val: T,
    pub(super) next: Option<NonNull<Node<T>>>,
    pub(super) prev: Option<NonNull<Node<T>>>,
    // _boo:PhantomData<T>
}
impl<T> Node<T> {
    fn new(val: T) -> Self {
        Self {
            val,
            next: None,
            prev: None,
        }
    }
}
pub(crate) struct Dlinklist<T> {
    pub(super) head: Option<NonNull<Node<T>>>,
    pub(super) tail: Option<NonNull<Node<T>>>,
    pub(super) size: usize,
    // _boo:PhantomData<T>
}
impl<T> Dlinklist<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            size: 0,
        }
    }
    pub fn push_back(&mut self, val: T) {
        //SAFETY : Box doesn't give us NULL pointer.
        let mut node = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(val)))) };
        if self.size == 0 {
            self.head = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
            self.tail = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
        } else {
            if let Some(mut last_node) = self.tail {
                // SAFETY : first_node is not NULL as size>0 that means head is pointing to a node.
                unsafe { last_node.as_mut() }.next =
                    Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
                unsafe { node.as_mut() }.prev =
                    Some(unsafe { NonNull::new_unchecked(last_node.as_ptr()) });
            }
            self.tail = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
        }
        self.size += 1;
    }
    pub fn pop_back(&mut self) {
        if self.size == 0 {
            return;
        }

        if self.size == 1 {
            let mut _eviction_node = self.tail;
            self.head = None;
            self.tail = None;
            if let Some(eviction_node_ptr) = _eviction_node {
                // SAFETY: size>=1 so tail is not NULL. i.e. eviction_node_ptr is also not NULL.
                let _ = unsafe { Box::from_raw(eviction_node_ptr.as_ptr()) };
            }
            _eviction_node = None;
        }
        if self.size > 1 {
            let mut _eviction_node = self.tail;
            if let Some(mut eviction_node_ptr) = _eviction_node {
                //SAFETY : As size==1 tail is not NULL, i.e. eviction node is not NULL.
                self.tail = unsafe { eviction_node_ptr.as_ref().prev };
                unsafe { eviction_node_ptr.as_mut() }.prev = None;
                let _ = unsafe { Box::from_raw(eviction_node_ptr.as_ptr()) };
                _eviction_node = None;
            }
            if let Some(mut new_last_node) = self.tail {
                //SAFETY : As size>1 tail.prev is also not NUL
                unsafe { new_last_node.as_mut() }.next = None;
            }
        }
        self.size -= 1;
    }
    pub fn push_front(&mut self, val: T) {
        //SAFETY : Box doesn't give us NULL pointer.
        let mut node = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(val)))) };
        if self.size == 0 {
            self.head = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
            self.tail = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
        } else {
            if let Some(mut first_node) = self.head {
                // SAFETY : first_node is not NULL as size>0 that means head is pointing to a node.
                unsafe { first_node.as_mut() }.prev =
                    Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
                unsafe { node.as_mut() }.next =
                    Some(unsafe { NonNull::new_unchecked(first_node.as_ptr()) });
            }
            self.head = Some(unsafe { NonNull::new_unchecked(node.as_ptr()) });
        }
        self.size += 1;
    }
    pub fn pop_front(&mut self) {
        if self.size == 0 {
            return;
        }
        if self.size == 1 {
            let mut _eviction_node = self.head;
            self.head = None;
            self.tail = None;
            if let Some(eviction_node_ptr) = _eviction_node {
                //SAFETY : As size==1 head is not NULL, i.e. eviction node is not NULL.
                let _ = unsafe { Box::from_raw(eviction_node_ptr.as_ptr()) };
            }
        }
        if self.size > 1 {
            let mut _eviction_node = self.head;
            if let Some(mut eviction_node_ptr) = _eviction_node {
                //SAFETY : As size >1 head is not NULL, i.e. eviction node is not NULL.
                self.head = unsafe { eviction_node_ptr.as_ref().next };
                unsafe { eviction_node_ptr.as_mut() }.next = None;
                let _ = unsafe { Box::from_raw(eviction_node_ptr.as_ptr()) };
                _eviction_node = None;
            }
            if let Some(mut new_first_node) = self.head {
                //SAFETY : As size>1 head.next is also not NULL
                unsafe { new_first_node.as_mut() }.prev = None;
            }
        }
        self.size -= 1;
    }
    pub fn erase(&mut self, mut node_ptr: NonNull<Node<T>>) {
        if self.size == 0 {
            return;
        }
        if self.head.unwrap() == node_ptr {
            self.pop_front();
            return;
        }
        if self.tail.unwrap() == node_ptr {
            self.pop_back();
            return;
        }
        let mut _prev_node = unsafe { node_ptr.as_ref() }.prev;
        let mut _next_node = unsafe { node_ptr.as_ref() }.next;
        if let Some(mut prev_node_ptr) = _prev_node {
            unsafe { prev_node_ptr.as_mut() }.next = _next_node;
            if let Some(mut next_node_ptr) = _next_node {
                unsafe { next_node_ptr.as_mut() }.prev = _prev_node;
            }
            _next_node = None;
        }
        _prev_node = None;
        unsafe { node_ptr.as_mut() }.next = None;
        unsafe { node_ptr.as_mut() }.prev = None;
        let _ = unsafe { Box::from_raw(node_ptr.as_ptr()) };
        self.size -= 1;
    }
}

impl<T: std::fmt::Display> Dlinklist<T> {
    pub fn print_list(&self) {
        if self.size == 0 {
            println!("Empty List.");
        }

        let mut cur = self.head;
        loop {
            if let Some(cur_node) = cur {
                print!("{} ", unsafe { cur_node.as_ref() }.val);
                cur = unsafe { cur_node.as_ref() }.next;
            } else {
                break;
            }
        }
        print!("\n");
    }
}
impl<T: Copy> Dlinklist<T> {
    pub fn front(&self) -> Option<T> {
        if self.size > 0 {
            return Some(unsafe { self.head.unwrap().as_ref() }.val);
        }
        return None;
    }
    pub fn back(&self) -> Option<T> {
        if self.size > 0 {
            return Some(unsafe { self.tail.unwrap().as_ref().val });
        }
        return None;
    }
}
impl<T: Clone> Dlinklist<T> {
    pub fn front_clone(&self) -> Option<T> {
        if self.size > 0 {
            return Some(unsafe { self.head.unwrap().as_ref() }.val.clone());
        }
        return None;
    }
    pub fn back_clone(&self) -> Option<T> {
        if self.size > 0 {
            return Some(unsafe { self.tail.unwrap().as_ref().val.clone() });
        }
        return None;
    }
}
impl<T> Drop for Dlinklist<T> {
    fn drop(&mut self) {
        while self.size > 0 {
            self.pop_back();
        }
    }
}
unsafe impl<T> Send for Dlinklist<T> {}
unsafe impl<T> Sync for Dlinklist<T> {}
unsafe impl<T> Send for Node<T> {}
unsafe impl<T> Sync for Node<T> {}
