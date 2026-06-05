use core::mem::MaybeUninit;

/// 固定容量环形缓冲区（Ring Buffer / Circular Queue）
///
/// 使用 `MaybeUninit` 避免不必要的初始化开销，
/// 用 `count` 字段区分「满」与「空」——这是环形缓冲区最经典的做法。
///
/// # 约束
/// - `N` 必须 ≥ 1。空队列和满队列通过 `count` 区分，不需要留一个空位。
pub struct Queue<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T, const N: usize> Queue<T, N> {
    /// 创建空队列
    pub fn new() -> Self {
        // `MaybeUninit::uninit().assume_init()` 分配一块未初始化的 `[MaybeUninit<T>; N]`，
        // 这是安全的，因为 `MaybeUninit` 本身就代表「可能未初始化」，
        // 我们严格追踪哪些槽位已初始化（count/head/tail）
        Queue {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// 入队。满则返回 `Err(item)`，元素不会入队
    pub fn enqueue(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }
        self.buf[self.tail] = MaybeUninit::new(item);
        self.tail = (self.tail + 1) % N;
        self.count += 1;
        Ok(())
    }

    /// 出队。空则返回 `None`
    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        // SAFETY: count > 0 保证此槽位已初始化
        let item = unsafe { self.buf[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.count -= 1;
        Some(item)
    }

    /// 查看队首元素（不可变引用）
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        // SAFETY: count > 0 保证此槽位已初始化
        Some(unsafe { self.buf[self.head].assume_init_ref() })
    }

    /// 查看队首元素（可变引用）
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.buf[self.head].assume_init_mut() })
    }

    /// 队列是否已满
    pub fn is_full(&self) -> bool {
        self.count == N
    }

    /// 队列是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// 当前元素个数
    pub fn len(&self) -> usize {
        self.count
    }

    /// 队列容量（固定为 `N`）
    pub const fn capacity(&self) -> usize {
        N
    }

    /// 清空队列，依次 drop 所有剩余元素
    pub fn clear(&mut self) {
        while self.dequeue().is_some() {}
    }

    /// 强制入队：满则覆盖最旧元素，返回被挤出的那个元素
    pub fn force_enqueue(&mut self, item: T) -> Option<T> {
        let dropped = if self.is_full() {
            self.dequeue() // 队列满时 dequeue 一定返回 Some
        } else {
            None
        };
        // 此时至少有一个空位，enqueue 不会失败
        let _ = self.enqueue(item);
        dropped
    }
}

impl<T, const N: usize> Drop for Queue<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}