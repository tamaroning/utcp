/// Fixed-length queue that drops the oldest element when it is full.
#[derive(Debug)]
pub struct SmallQueue<T, const N: usize> {
    buf: [T; N],
    len: usize,
    head: usize,
    tail: usize,
}

impl<T: Default, const N: usize> SmallQueue<T, N> {
    pub fn new() -> Self {
        Self {
            buf: std::array::from_fn(|_| Default::default()),
            len: 0,
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, elem: T) -> Option<()> {
        if self.len == self.buf.len() {
            self.pop_front();
        }
        self.buf[self.tail] = elem;
        self.tail = (self.tail + 1) % self.buf.len();
        self.len += 1;
        Some(())
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let elem = core::mem::replace(&mut self.buf[self.head], Default::default());
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        Some(elem)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

#[test]
fn test_small_queue() {
    let mut q = SmallQueue::<i32, 3>::new();
    assert_eq!(q.pop_front(), None);
    q.push(1);
    q.push(2);
    q.push(3);
    q.push(4);
    assert_eq!(q.pop_front(), Some(2));
    assert_eq!(q.pop_front(), Some(3));
    assert_eq!(q.pop_front(), Some(4));
    assert_eq!(q.pop_front(), None);
    q.push(5);
    q.push(6);
    assert_eq!(q.pop_front(), Some(5));
    assert_eq!(q.pop_front(), Some(6));
    assert_eq!(q.pop_front(), None);
    q.push(7);
    q.push(8);
    q.push(9);
    q.push(10);
    q.push(11);
    q.push(12);
    q.push(13);
    assert_eq!(q.pop_front(), Some(11));
    assert_eq!(q.pop_front(), Some(12));
    assert_eq!(q.pop_front(), Some(13));
}

#[test]
fn test_small_queue2() {
    let mut q = SmallQueue::<(u16, Vec<u8>), 3>::new();
    q.push((1, b"Hello, World".to_vec()));
    q.push((2, b"Hello, Rust".to_vec()));
    assert_eq!(q.pop_front(), Some((1, b"Hello, World".to_vec())));
    assert_eq!(q.pop_front(), Some((2, b"Hello, Rust".to_vec())));
}
