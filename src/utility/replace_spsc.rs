use std::cell::Cell;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::*;

// ////////////////////////////////////////////////////////
// Internal
// ////////////////////////////////////////////////////////

const BUFFER_SIZE: usize = 16;

/// The internal memory buffer used by the replace spsc. It's unlikely, but during a read, a write
/// could happen inbetween the atomic load and the dereference. This is unlikely because 16 writes
/// would have to happen during that time.
struct Buffer<T: Copy> {
    read: AtomicPtr<T>,
    current: Cell<usize>,
    write: [T; BUFFER_SIZE],
}

unsafe impl<T: Copy + Sync> Sync for Buffer<T> {}

impl<T: Copy> Buffer<T> {
    fn new() -> Buffer<T> {
        Buffer {
            read: AtomicPtr::new(0 as *mut T),
            current: Cell::new(0),
            write: unsafe { mem::uninitialized() },
        }
    }

    pub fn read(&self) -> T {
        // It's unlikely, but a write could happen inbetween the atomic load and the dereference.
        // This is unlikely because 16 writes would have to happen during that time.
        unsafe { *self.read.load(Ordering::Acquire) }
    }

    pub fn write(&self, value: T) {
        let x = self.current.get();
        unsafe {
            let pointer = self.write.as_ptr().wrapping_add(x) as *mut T;
            *pointer = value;
            self.read.store(pointer, Ordering::Release);
        }
        self.current.set((x + 1) & (BUFFER_SIZE - 1));
    }
}

pub fn make<T: Copy>(initial: T) -> (Producer<T>, Consumer<T>) {
    // This is the only place where a buffer is created.
    let arc = Arc::new(Buffer::new());
    (*arc).write(initial);
    (Producer { buffer: arc.clone() }, Consumer { buffer: arc.clone() })
}

// ////////////////////////////////////////////////////////
// Public
// ////////////////////////////////////////////////////////

/// A handle which allows adding values onto the buffer.
pub struct Producer<T: Copy> {
    buffer: Arc<Buffer<T>>,
}

/// A handle which allows consuming values from the buffer.
pub struct Consumer<T: Copy> {
    buffer: Arc<Buffer<T>>,
}

unsafe impl<T: Copy + Send> Send for Producer<T> {}
unsafe impl<T: Copy + Send> Send for Consumer<T> {}

impl<T> !Sync for Producer<T> {}
impl<T> !Sync for Consumer<T> {}

impl<T: Copy> Producer<T> {
    /// Push a value onto the buffer.
    ///
    /// If the buffer is non-full, the operation will execute immediately. If the buffer is
    /// populated, this operation overwrites the stored value. If the buffer is contested by a
    /// read from the consumer, it will spin until the read is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate storm;
    /// use storm::utility::replace_spsc::*;
    ///
    /// let (producer, _) = make(0u32);
    ///
    /// producer.set(1u32);
    /// ```
    pub fn set(&self, value: T) {
        (*self.buffer).write(value);
    }
}

impl<T: Copy> Consumer<T> {
    /// Attempt to pop a value from the buffer.
    ///
    /// This method does not block.  If the buffer is empty, the method will return `None`. If
    /// there is a value available, the method will return `Some(v)`, where `v` is the value being
    /// consumed from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate storm;
    /// use storm::utility::replace_spsc::*;
    ///
    /// let (_, consumer) = make(1u32);
    ///
    /// let t = consumer.get();
    /// ```
    pub fn get(&self) -> T {
        (*self.buffer).read()
    }
}
