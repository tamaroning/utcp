/// Fixed-length queue that drops the oldest element when full
#[derive(Debug)]
pub struct SmallQueue<T, const N: usize> {
    buf: [T; N],
    len: usize,
    head: usize,
    tail: usize,
}

impl<T, const N: usize> SmallQueue<T, N> {
    pub fn new() -> Self {
        Self {
            buf: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
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
        let elem = core::mem::replace(&mut self.buf[self.head], unsafe {
            core::mem::MaybeUninit::uninit().assume_init()
        });
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
}
