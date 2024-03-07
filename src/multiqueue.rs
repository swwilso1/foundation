//! The `multiqueue` module provides the `MultiQueue` object which is a queue that allows for
//! safety when used between threads and for forking the queue to create a new queue that shares
//! the same underlying data.

use crate::error::FoundationError;
use log::error;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// This module makes use of raw pointers and unsafe code to implement the container structure.
// Normally, we would use a safe pre-existing Rust container, but for speed and correctness, we
// actually need to use the raw pointers. We wrap the unsafe code in a safe interface and provide
// internal assertions and checks to make sure we use the pointers correctly (YMMV).

/// The `Block` struct is a node in the queue that contains the object to be stored in the queue,
/// The queue is implemented with a singly linked list with the `Block` struct as the basic node
/// in the list.
struct Block<T> {
    // A pointer to the next block in the list.
    next: *mut Block<T>,

    // The data contained in the block.
    object: T,

    // The reference count of the block.
    reference_count: u32,
}

impl<T> Block<T> {
    /// The `new` function creates a new `Block` object with the given object.
    ///
    /// # Arguments
    ///
    /// * `object` - The object to store in the block.
    ///
    /// # Returns
    ///
    ///
    fn new(object: T) -> Block<T> {
        Block {
            next: std::ptr::null_mut(),
            object,
            reference_count: 1,
        }
    }
}

/// The `Core` struct is the underlying data structure for the `MultiQueue` object. It contains
/// the linked list of blocks and a reference count for the core. In this object we use the
/// reference count to know when to drop the blocks from the linked list. The reference counting
/// for the `Core` object happens in an `Arc<Core>` wrapper.
pub struct Core<T> {
    /// A pointer to the first block in the queue.
    head: *mut Block<T>,

    /// A pointer to the last block in the queue.
    tail: *mut Block<T>,

    /// The reference count of the core.
    reference_count: u32,
}

impl<T> Core<T> {
    /// The `new` function creates a new `Core` object.
    ///
    /// # Returns
    ///
    ///
    pub fn new() -> Core<T> {
        Core {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            reference_count: 1,
        }
    }

    /// The `push_back` function adds an object to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `object` - The object to add to the back of the queue.
    pub fn push_back(&mut self, object: T) {
        // The block memory must be created with the `Box` allocator, so we can use
        // the `Box` deallocator to drop the block when it is no longer needed.
        let block = Box::new(Block::new(object));
        let raw = Box::into_raw(block);

        if self.head.is_null() {
            // Insert the new block as the first block in the queue.
            self.head = raw;
            self.tail = raw;
        } else {
            assert_eq!(self.tail.is_null(), false, "tail is null");
            unsafe {
                // Insert the new block after the current tail.
                (*self.tail).next = raw;
            }

            // Make the new block the new tail.
            self.tail = raw;
        }

        assert_eq!(self.tail.is_null(), false, "tail is null");

        unsafe {
            // The block gets the current number of references as there are references
            // to the `Core` object.
            (*self.tail).reference_count = self.reference_count;
        }
    }

    /// The `update` function removes any blocks from the front of the queue that have a reference
    /// count of 0.
    pub fn update(&mut self) {
        // Start looking from the head of the queue.
        let mut tmp = self.head;
        let mut previous: *mut Block<T> = std::ptr::null_mut();

        while tmp != std::ptr::null_mut() {
            unsafe {
                if (*tmp).reference_count == 0 {
                    // If the block we are examining is the last block, then make the last block
                    // point to whatever is next after the current block (probably null, but
                    // not necessarily).
                    if self.tail == tmp {
                        self.tail = (*tmp).next;
                    }

                    // We are keeping track of the previous node in the list. This does allow
                    // us to remove a node from the middle of the list. It is a bit uncertain
                    // if we can actually have a node with a zero reference count in the middle
                    // of the list.
                    if previous != std::ptr::null_mut() {
                        (*previous).next = (*tmp).next;
                        // This drop removes the block from the list and drops the memory. We must
                        // use the Box wrapper to remove the memory from the heap.
                        drop(Box::from_raw(tmp));
                        tmp = (*previous).next;
                    } else {
                        self.head = (*tmp).next;
                        // This drop removes the block from the list and drops the memory. We must
                        // use the Box wrapper to remove the memory from the heap.
                        drop(Box::from_raw(tmp));
                        tmp = self.head;
                    }
                } else {
                    previous = tmp;
                    tmp = (*tmp).next;
                }
            }
        }

        if self.tail.is_null() {
            // Now set the tail pointer to the correct block.
            if self.size() == 1 {
                self.tail = self.head;
            } else {
                tmp = self.head;
                while tmp != std::ptr::null_mut() {
                    unsafe {
                        self.tail = tmp;
                        tmp = (*tmp).next;
                    }
                }
            }
        }
    }

    /// The `size` function returns the number of elements in the queue.
    ///
    /// # Returns
    ///
    /// The number of elements in the queue.
    pub fn size(&self) -> usize {
        let mut count = 0;
        let mut tmp = self.head;
        while tmp != std::ptr::null_mut() {
            count += 1;
            unsafe {
                tmp = (*tmp).next;
            }
        }
        count
    }

    /// The `empty` function returns true if the queue is empty.
    pub fn empty(&self) -> bool {
        self.head.is_null()
    }
}

/// The `MultiQueue` struct is a queue that allows for safety when used between threads and for
/// forking the queue to create a new queue that shares the same underlying data.
pub struct MultiQueue<T> {
    /// The shared core object of the queue. (shared between queue forks)
    core: Arc<Mutex<Core<T>>>,

    /// A pointer to the first block in the queue.
    head: *mut Block<T>,

    /// A flag to indicate if we are at the end of the queue. We need this flag in the case
    /// that the queue is forked before we insert any elements to help correctly keep track
    /// of the block reference counts.
    at_end_of_queue: bool,
}

impl<T> MultiQueue<T> {
    /// The `new` function creates a new `MultiQueue` object.
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            core: Arc::new(Mutex::new(Core::new())),
            head: std::ptr::null_mut(),
            at_end_of_queue: false,
        }
    }

    /// The `push_back` function adds an object to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `object` - The object to add to the back of the queue.
    ///
    /// # Returns
    ///
    /// An `Ok` result if the object was added to the queue, otherwise a `FoundationError`.
    pub fn push_back(&mut self, object: T) -> Result<(), FoundationError> {
        match self.core.lock() {
            Ok(mut core) => {
                core.push_back(object);
                if self.head == std::ptr::null_mut() {
                    self.head = core.head;
                }
                Ok(())
            }
            Err(e) => Err(FoundationError::OperationFailed(format!(
                "Could not lock the MultiQueue core: {}",
                e
            ))),
        }
    }

    /// The `empty` function returns true if the queue is empty.
    pub fn empty(&self) -> bool {
        match self.core.lock() {
            Ok(core) => {
                if self.head == std::ptr::null_mut() {
                    return core.empty();
                }

                if self.at_end_of_queue {
                    // We just verified that self.head points to something.
                    unsafe {
                        return (*self.head).next.is_null();
                    }
                }

                false
            }
            Err(_) => {
                error!("Could not lock the MultiQueue core");
                true
            }
        }
    }

    /// The `front` function returns a reference to the object at the front of the queue.
    ///
    /// # Returns
    ///
    /// A reference to the object at the front of the queue, or `None` if the queue is empty.
    pub fn front(&mut self) -> Option<&T> {
        match self.core.lock() {
            Ok(mut core) => {
                if core.empty() {
                    return None;
                }

                if self.head == std::ptr::null_mut() {
                    self.head = core.head;
                }

                if self.at_end_of_queue {
                    // We just verified that self.head points to something valid.
                    let next = unsafe { (*self.head).next };

                    if next == std::ptr::null_mut() {
                        return None;
                    }

                    unsafe {
                        (*self.head).reference_count -= 1;
                    }

                    core.update();

                    self.head = next;
                    self.at_end_of_queue = false;
                }

                assert_eq!(self.head.is_null(), false, "head is null");
                unsafe {
                    return Some(&(*self.head).object);
                }
            }
            Err(_) => {
                error!("Could not lock the MultiQueue core");
                None
            }
        }
    }

    /// The `front_mut` function returns a mutable reference to the object at the front of the queue.
    ///
    /// # Returns
    ///
    /// A mutable reference to the object at the front of the queue, or `None` if the queue is empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        match self.core.lock() {
            Ok(mut core) => {
                if core.empty() {
                    return None;
                }

                if self.head == std::ptr::null_mut() {
                    self.head = core.head;
                }

                if self.at_end_of_queue {
                    // We just verified that self.head points to something valid.
                    let next = unsafe { (*self.head).next };

                    if next == std::ptr::null_mut() {
                        return None;
                    }

                    unsafe {
                        (*self.head).reference_count -= 1;
                    }

                    core.update();

                    self.head = next;
                    self.at_end_of_queue = false;
                }

                assert_eq!(self.head.is_null(), false, "head is null");
                unsafe {
                    return Some(&mut (*self.head).object);
                }
            }
            Err(_) => {
                error!("Could not lock the MultiQueue core");
                None
            }
        }
    }

    /// The `pop_front` function removes the object at the front of the queue.
    /// If the queue is empty, then this function does nothing.
    pub fn pop_front(&mut self) {
        match self.core.lock() {
            Ok(mut core) => {
                if core.empty() {
                    return;
                }

                if self.head == std::ptr::null_mut() {
                    self.head = core.head;
                }

                let next = unsafe { (*self.head).next };

                if next != std::ptr::null_mut() {
                    if self.at_end_of_queue {
                        self.at_end_of_queue = false;
                    }

                    unsafe {
                        (*self.head).reference_count -= 1;
                    }
                    self.head = next;
                    core.update();
                } else {
                    self.at_end_of_queue = true;
                }
            }
            Err(e) => {
                error!("Could not lock the MultiQueue core: {}", e);
            }
        }
    }

    /// The `fork` function creates a new `MultiQueue` object that shares the same underlying data
    /// as the original queue.
    ///
    /// # Returns
    ///
    /// A new `MultiQueue` object that shares the same underlying data as the original queue or a
    /// `FoundationError` if the fork operation failed.
    pub fn fork(&mut self) -> Result<MultiQueue<T>, FoundationError> {
        match self.core.lock() {
            Ok(mut core) => {
                // Update the reference counts of the blocks in the queue before we create
                // the new queue structure.
                core.reference_count += 1;
                let mut tmp = self.head;
                while tmp != std::ptr::null_mut() {
                    unsafe {
                        (*tmp).reference_count += 1;
                        tmp = (*tmp).next;
                    }
                }
            }
            Err(e) => {
                return Err(FoundationError::OperationFailed(format!(
                    "Could not lock the MultiQueue core: {}",
                    e
                )));
            }
        }

        Ok(MultiQueue {
            core: self.core.clone(),
            head: self.head,
            at_end_of_queue: self.at_end_of_queue,
        })
    }

    /// The `size` function returns the number of elements in the queue.
    /// If an error occurs while locking the core, then this function returns 0.
    ///
    /// # Returns
    ///
    ///
    pub fn size(&self) -> usize {
        match self.core.lock() {
            Ok(core) => {
                if core.empty() {
                    return 0;
                }

                if self.at_end_of_queue {
                    if self.head == std::ptr::null_mut() {
                        return core.size();
                    }

                    unsafe {
                        return self.count_size_from((*self.head).next);
                    }
                }

                let tmp = if self.head == std::ptr::null_mut() {
                    core.head
                } else {
                    self.head
                };
                self.count_size_from(tmp)
            }
            Err(_) => {
                error!("Could not lock the MultiQueue core");
                0
            }
        }
    }

    /// The `references` function returns the number of references to the core of the queue.
    /// If an error occurs while locking the core, then this function returns 0.
    pub fn references(&self) -> u32 {
        match self.core.lock() {
            Ok(core) => core.reference_count,
            Err(_) => {
                error!("Could not lock the MultiQueue core");
                0
            }
        }
    }

    /// The `count_size_from` function returns the number of elements in the queue starting from
    /// the given block.
    fn count_size_from(&self, block: *mut Block<T>) -> usize {
        let mut count = 0;
        let mut tmp = block;
        while tmp != std::ptr::null_mut() {
            count += 1;
            unsafe {
                tmp = (*tmp).next;
            }
        }
        count
    }

    /// The `iter` function returns an iterator over the elements in the queue.
    pub fn iter(&mut self) -> MultiQueueIterator<T> {
        MultiQueueIterator::new(self)
    }
}

impl<T> Drop for MultiQueue<T> {
    fn drop(&mut self) {
        match self.core.lock() {
            Ok(mut core) => {
                // Decrement the reference count of the core. We do not actually
                // delete the core because the Arc around the core will handle that
                // deletion. We are just keeping the reference counting that handles
                // the blocks up-to-date.
                core.reference_count -= 1;
            }
            Err(_) => {
                error!("Could not lock the MultiQueue core");
            }
        }
    }
}

// We provide Send + Sync implementation for MultiQueue so that we can move a MultiQueue to
// a different thread or async execution. We take care to make sure the pointer usage in the
// MultiQueue is all heap based and not thread specific or stack based.
unsafe impl<T> Send for MultiQueue<T> {}
unsafe impl<T> Sync for MultiQueue<T> {}

pub struct MultiQueueIterator<'a, T> {
    head: *mut Block<T>,

    // Our iterator does not contain a reference to the core, but rather a pointer, so we use
    // the PhantomData member to ensure that the pointer has the same lifetime as the core.
    phantom: PhantomData<&'a T>,
}

impl<'a, T> MultiQueueIterator<'a, T> {
    pub fn new(queue: &'a mut MultiQueue<T>) -> MultiQueueIterator<'a, T> {
        MultiQueueIterator {
            head: queue.head,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for MultiQueueIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Just a reminder here that the head pointer is not actually the head inside the queue,
        // but rather our head pointer that we copied from the queue. (I include this comment
        // because it helped me to remember what was going on here.)
        if self.head == std::ptr::null_mut() {
            return None;
        }

        unsafe {
            let result = Some(&(*self.head).object);
            self.head = (*self.head).next;
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::threadpool::{ThreadJob, ThreadPool};

    #[test]
    fn test_multiqueue() {
        let mut queue = MultiQueue::new();
        assert_eq!(queue.empty(), true);
        assert_eq!(queue.size(), 0);
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();
        assert_eq!(queue.empty(), false);
        assert_eq!(queue.size(), 3);
        assert_eq!(queue.front(), Some(&1));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&2));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&3));
        queue.pop_front();
        assert_eq!(queue.empty(), true);
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn test_push_back() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();
        queue.push_back(4).unwrap();
        assert_eq!(queue.size(), 4);
        assert_eq!(queue.front(), Some(&1));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&2));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&3));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&4));
        queue.pop_front();
        assert_eq!(queue.empty(), true);
    }

    #[test]
    fn test_empty() {
        let mut queue = MultiQueue::new();
        assert_eq!(queue.empty(), true);
        queue.push_back(1).unwrap();
        assert_eq!(queue.empty(), false);
        queue.pop_front();
        assert_eq!(queue.empty(), true);
    }

    #[test]
    fn test_front() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();
        assert_eq!(queue.front(), Some(&1));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&2));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&3));
        queue.pop_front();
        assert_eq!(queue.front(), None);
    }

    #[test]
    fn test_front_mut() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();
        assert_eq!(queue.front_mut(), Some(&mut 1));
        queue.pop_front();
        assert_eq!(queue.front_mut(), Some(&mut 2));
        queue.pop_front();
        assert_eq!(queue.front_mut(), Some(&mut 3));
        *queue.front_mut().unwrap() = 4;
        assert_eq!(queue.front(), Some(&4));
        queue.pop_front();
        assert_eq!(queue.front_mut(), None);
    }

    #[test]
    fn test_pop_front() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();
        queue.pop_front();
        assert_eq!(queue.front(), Some(&2));
        queue.pop_front();
        assert_eq!(queue.front(), Some(&3));
        queue.pop_front();
        assert_eq!(queue.front(), None);
    }

    #[test]
    fn test_size() {
        let mut queue = MultiQueue::new();
        assert_eq!(queue.size(), 0);
        queue.push_back(1).unwrap();
        assert_eq!(queue.size(), 1);
        queue.push_back(2).unwrap();
        assert_eq!(queue.size(), 2);
        queue.push_back(3).unwrap();
        assert_eq!(queue.size(), 3);
        queue.pop_front();
        assert_eq!(queue.size(), 2);
        queue.pop_front();
        assert_eq!(queue.size(), 1);
        queue.pop_front();
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn test_basic_fork() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        let mut fork = queue.fork().unwrap();

        assert_eq!(queue.size(), 3);
        assert_eq!(fork.size(), 3);

        queue.pop_front();

        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 3);

        fork.pop_front();

        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 2);

        fork.pop_front();

        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 1);

        fork.pop_front();

        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 0);

        fork.pop_front();

        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 0);

        queue.pop_front();

        assert_eq!(queue.size(), 1);
        assert_eq!(fork.size(), 0);

        queue.pop_front();

        assert_eq!(queue.size(), 0);
        assert_eq!(fork.size(), 0);
    }

    #[test]
    fn test_contents_with_fork() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        let mut fork = queue.fork().unwrap();

        assert_eq!(queue.front(), Some(&1));
        assert_eq!(fork.front(), Some(&1));

        queue.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&1));

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&2));

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&3));

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), None);

        queue.pop_front();

        assert_eq!(queue.front(), Some(&3));
        assert_eq!(fork.front(), None);

        queue.pop_front();

        assert_eq!(queue.front(), None);
        assert_eq!(fork.front(), None);
    }

    #[test]
    fn test_mutable_element_with_fork() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        let mut fork = queue.fork().unwrap();

        assert_eq!(queue.front(), Some(&1));
        assert_eq!(fork.front(), Some(&1));

        *queue.front_mut().unwrap() = 4;

        assert_eq!(queue.front(), Some(&4));
        assert_eq!(fork.front(), Some(&4));

        queue.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&4));

        *fork.front_mut().unwrap() = 5;

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&5));

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&2));

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&3));

        *fork.front_mut().unwrap() = 6;

        queue.pop_front();

        assert_eq!(queue.front(), Some(&6));
        assert_eq!(fork.front(), Some(&6));

        queue.pop_front();

        assert_eq!(queue.front(), None);
        assert_eq!(fork.front(), Some(&6));

        fork.pop_front();

        assert_eq!(queue.front(), None);
        assert_eq!(fork.front(), None);
    }

    #[test]
    fn test_iterator() {
        let mut queue = MultiQueue::new();
        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        let mut iter = queue.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fork_references() {
        let mut queue = MultiQueue::new();
        let mut fork = queue.fork().unwrap();

        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        assert_eq!(queue.references(), 2);
        assert_eq!(fork.references(), 2);
        assert_eq!(queue.size(), 3);
        assert_eq!(fork.size(), 3);
        assert_eq!(queue.front(), Some(&1));
        assert_eq!(fork.front(), Some(&1));

        fork.pop_front();

        assert_eq!(queue.size(), 3);
        assert_eq!(fork.size(), 2);
        assert_eq!(queue.front(), Some(&1));
        assert_eq!(fork.front(), Some(&2));

        queue.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&2));
        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 2);

        fork.pop_front();

        assert_eq!(queue.front(), Some(&2));
        assert_eq!(fork.front(), Some(&3));
        assert_eq!(queue.size(), 2);
        assert_eq!(fork.size(), 1);

        queue.pop_front();

        assert_eq!(queue.front(), Some(&3));
        assert_eq!(fork.front(), Some(&3));
        assert_eq!(queue.size(), 1);
        assert_eq!(fork.size(), 1);

        queue.pop_front();

        assert_eq!(queue.front(), None);
        assert_eq!(fork.front(), Some(&3));
        assert_eq!(queue.size(), 0);
        assert_eq!(fork.size(), 1);

        fork.pop_front();

        assert_eq!(queue.front(), None);
        assert_eq!(fork.front(), None);
        assert_eq!(queue.size(), 0);
        assert_eq!(fork.size(), 0);
    }

    const BUFFER_SIZE: usize = 8192;

    #[test]
    fn test_with_buffer() {
        let mut queue: MultiQueue<[u8; BUFFER_SIZE]> = MultiQueue::new();
        let mut fork = queue.fork().unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        for i in 0..BUFFER_SIZE {
            buffer[i] = i as u8;
        }
        queue.push_back(buffer).unwrap();
        fork.push_back(buffer).unwrap();
        let iter = queue.iter();
        let fork_iter = fork.iter();
        let mut count = 0;
        for (a, b) in iter.zip(fork_iter) {
            for i in 0..BUFFER_SIZE {
                assert_eq!(a[i], b[i]);
            }
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_multiqueue_in_tokio() {
        let mut queue = MultiQueue::new();

        queue.push_back(1).unwrap();
        queue.push_back(2).unwrap();
        queue.push_back(3).unwrap();

        let mut fork = queue.fork().unwrap();

        let handle1 = tokio::spawn(async move {
            assert_eq!(fork.front(), Some(&1));
            fork.pop_front();
            assert_eq!(fork.front(), Some(&2));
            fork.pop_front();
            assert_eq!(fork.front(), Some(&3));
        });

        let handle2 = tokio::spawn(async move {
            let mut iter = queue.iter();
            assert_eq!(iter.next(), Some(&1));
            assert_eq!(iter.next(), Some(&2));
            assert_eq!(iter.next(), Some(&3));
            queue.pop_front();
            queue.pop_front();
            queue.pop_front();
            assert_eq!(queue.size(), 0);
        });

        let handles = vec![handle1, handle2];

        futures::future::join_all(handles).await;
    }

    #[tokio::test]
    async fn test_multiqueue_thread_access() {
        let mut queue = MultiQueue::new();
        let mut fork = queue.fork().unwrap();

        let handle1 = tokio::spawn(async move {
            queue.push_back(1).unwrap();
            queue.push_back(2).unwrap();
            queue.push_back(3).unwrap();
        });

        let handle2 = tokio::spawn(async move {
            assert_eq!(fork.front(), Some(&1));
            fork.pop_front();
            assert_eq!(fork.front(), Some(&2));
            fork.pop_front();
            assert_eq!(fork.front(), Some(&3));
        });

        let handles = vec![handle1, handle2];

        futures::future::join_all(handles).await;
    }

    #[tokio::test]
    async fn test_with_threadpool() {
        let mut thread_pool = ThreadPool::new(4);
        let mut queue: MultiQueue<i32> = MultiQueue::new();
        let mut fork = queue.fork().unwrap();

        let finished = Arc::new(Mutex::new(false));
        let finished2 = finished.clone();

        let mut job1 = ThreadJob::new();
        job1.add_task(Box::pin(async move {
            // Load the queue.
            queue.push_back(1).unwrap();
            queue.push_back(2).unwrap();
            queue.push_back(3).unwrap();

            // Now drain our part of the queue.
            queue.pop_front();
            queue.pop_front();
            queue.pop_front();

            assert!(queue.empty());

            Ok(())
        }));

        let mut job2 = ThreadJob::new();
        job2.add_task(Box::pin(async move {
            while fork.empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            assert_eq!(fork.front(), Some(&1));
            fork.pop_front();

            while fork.empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            assert_eq!(fork.front(), Some(&2));
            fork.pop_front();

            while fork.empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            assert_eq!(fork.front(), Some(&3));
            fork.pop_front();
            assert_eq!(fork.size(), 0);
            assert!(fork.empty());

            *finished2.lock().unwrap() = true;

            Ok(())
        }));

        thread_pool.add_job(job1).unwrap();
        thread_pool.add_job(job2).unwrap();

        while !*finished.lock().unwrap() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_clone_with_threadpool() {
        let mut thread_pool = ThreadPool::new(4);
        let queue: Arc<Mutex<MultiQueue<i32>>> = Arc::new(Mutex::new(MultiQueue::new()));
        let fork = queue.clone();

        let finished = Arc::new(Mutex::new(false));
        let finished2 = finished.clone();

        let thread2_finished = Arc::new(Mutex::new(false));
        let thread2_finished2 = thread2_finished.clone();

        let mut job1 = ThreadJob::new();
        job1.add_task(Box::pin(async move {
            // Load the queue.
            if let Ok(mut queue) = queue.lock() {
                queue.push_back(1).unwrap();
                queue.push_back(2).unwrap();
                queue.push_back(3).unwrap();
            }
            // queue.push_back(1).unwrap();
            // queue.push_back(2).unwrap();
            // queue.push_back(3).unwrap();

            while *thread2_finished2.lock().unwrap() == false {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            if let Ok(queue) = queue.lock() {
                assert!(queue.empty());
            }

            // assert!(queue.empty());

            Ok(())
        }));

        let mut job2 = ThreadJob::new();
        job2.add_task(Box::pin(async move {
            let mut empty = fork.lock().unwrap().empty();
            while empty {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                empty = fork.lock().unwrap().empty();
            }

            if let Ok(mut fork) = fork.lock() {
                assert_eq!(fork.front(), Some(&1));
                fork.pop_front();
            }

            // assert_eq!(fork.front(), Some(&1));
            // fork.pop_front();

            empty = fork.lock().unwrap().empty();
            while empty {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                empty = fork.lock().unwrap().empty();
            }

            if let Ok(mut fork) = fork.lock() {
                assert_eq!(fork.front(), Some(&2));
                fork.pop_front();
            }
            // assert_eq!(fork.front(), Some(&2));
            // fork.pop_front();

            empty = fork.lock().unwrap().empty();
            while empty {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                empty = fork.lock().unwrap().empty();
            }

            if let Ok(mut fork) = fork.lock() {
                assert_eq!(fork.front(), Some(&3));
                fork.pop_front();
                assert_eq!(fork.size(), 0);
                assert!(fork.empty());
            }
            // assert_eq!(fork.front(), Some(&3));
            // fork.pop_front();
            // assert_eq!(fork.size(), 0);
            // assert!(fork.empty());

            *thread2_finished.lock().unwrap() = true;
            // Just give up the CPU so that the other thread can finish.  This is not super
            // deterministic.
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            *finished2.lock().unwrap() = true;

            Ok(())
        }));

        thread_pool.add_job(job1).unwrap();
        thread_pool.add_job(job2).unwrap();

        while !*finished.lock().unwrap() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        thread_pool.stop();
    }
}
