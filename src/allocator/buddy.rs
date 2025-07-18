use crate::println;

use super::Locked;
use core::alloc::{GlobalAlloc, Layout};

#[derive(Debug, Clone, Copy)]
enum Flag {
    Free,
    Taken,
}

#[derive(Debug)]
struct ListNode {
    flag: Flag,
    level: u8,
    next: Option<&'static mut ListNode>,
    prev: Option<&'static mut ListNode>,
}

const MIN: u8 = 5;
const LEVELS: u8 = 10;
const MIN_BYTES: usize = 2usize.pow(MIN as u32);
const MAX_BYTES: usize = 2usize.pow((MIN + LEVELS - 1) as u32);
const PAGE: u32 = 4096;

pub struct BuddyAllocator {
    list_heads: [Option<&'static mut ListNode>; LEVELS as usize],
}

impl Default for BuddyAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl BuddyAllocator {
    /// Creates an empty BuddyAllocator
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        BuddyAllocator {
            list_heads: [EMPTY; LEVELS as usize],
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given heap bounds are valid
    /// and that the heap is unused. This method must be called only onec.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        let mut heap_left = heap_size;
        let mut ptr = heap_start;
        for level in (MIN..(LEVELS - 1)).rev() {
            let block_size = 2usize.pow((MIN + level) as u32);
            while heap_left > block_size {
                println!("Creating level {} size {} at {:X}", level, block_size, ptr);
                let node = ptr as *mut ListNode;
                if let Some(old_node) = self.list_heads[level as usize].take() {
                    unsafe {
                        old_node.prev = Some(&mut *node);
                        (*node).next = Some(old_node);
                    }
                } else {
                    unsafe {
                        (*node).next = None;
                    }
                }
                unsafe {
                    (*node).level = level;
                    (*node).flag = Flag::Free;
                    (*node).prev = None;
                }

                self.list_heads[level as usize] = unsafe { Some(&mut *node) };

                heap_left -= block_size;
                ptr += block_size;
            }
        }
    }
}

impl ListNode {
    fn buddy(&mut self) -> *mut Self {
        let index = self.level;
        let mask = 0x1 << (index + MIN);
        (self as *mut ListNode as usize ^ mask) as *mut ListNode
    }

    fn split(&mut self) -> *mut Self {
        let index = self.level - 1;
        let mask = 0x1 << (index + MIN) as usize;
        (self as *mut ListNode as usize | mask) as *mut ListNode
    }

    fn primary(&mut self) -> *mut Self {
        let index = self.level - 1;
        // let mask = 0xffffffffffffffff << (1 + index + MIN);
        let mask = -1 << (1 + index + MIN);
        (self as *mut ListNode as i32 & mask) as *mut ListNode
    }

    fn hide(&mut self, layout: Layout) -> *mut u8 {
        let size = core::cmp::max(layout.align(), size_of::<ListNode>());
        (self as *mut ListNode as *mut u8).wrapping_add(size)
    }

    fn magic(mem: *mut u8, layout: Layout) -> *mut Self {
        let size = core::cmp::max(layout.align(), size_of::<ListNode>());
        mem.wrapping_sub(size) as *mut ListNode
    }
}

fn level(req: usize) -> usize {
    let total = req + 24;

    let mut i = 0;
    let mut size = 1 << MIN;
    while total > size {
        size <<= 1;
        i += 1;
    }

    i
}

impl BuddyAllocator {
    fn find(&mut self, level: usize) -> *mut u8 {
        match self.list_heads[level].take() {
            Some(node) => {
                self.list_heads[level] = node.next.take();
                node.prev = None;
                node.flag = Flag::Taken;
                node as *mut ListNode as *mut u8
            }
            None => {
                let mut found_level = level + 1;
                for current in (level + 1)..(LEVELS as usize) {
                    if let Some(node) = self.list_heads[current].take() {
                        self.list_heads[current] = Some(node);
                        found_level += 1;
                        break;
                    }

                    found_level += 1;
                }

                for current in ((level + 1)..found_level).rev() {
                    match self.list_heads[current].take() {
                        Some(node) => {
                            self.list_heads[current] = node.next.take();
                            let buddy = (*node).split();

                            node.level = (current - 1) as u8;

                            unsafe { (*buddy).level = (current - 1) as u8 };
                            unsafe { (*buddy).flag = Flag::Free };
                            unsafe { (*buddy).next = None };
                            unsafe { (*buddy).prev = None };

                            node.next = unsafe { Some(&mut *buddy) };

                            self.list_heads[current - 1] = Some(node);
                        }
                        None => {
                            panic!("fault in splitng of memory blocks in buddy allocator")
                        }
                    }
                }

                if let Some(node) = self.list_heads[level].take() {
                    self.list_heads[level] = node.next.take();
                    node.prev = None;
                    node.flag = Flag::Taken;
                    node as *mut ListNode as *mut u8
                } else {
                    panic!("fault in spliting of memory blocks in buddy allocator");
                }
            }
        }
    }

    fn insert(&mut self, node: *mut ListNode) {
        let buddy = unsafe { (*node).buddy() };

        match unsafe { (*buddy).flag } {
            Flag::Free => {
                if unsafe { (*buddy).level == (*node).level } {
                    unsafe { (*node).level += 1 };
                }
            }
            Flag::Taken => {}
        }
        let level = unsafe { (*node).level };
        unsafe { (*node).flag = Flag::Free }
        match self.list_heads[level as usize].take() {
            Some(prev) => unsafe { (*node).next = Some(prev) },
            None => self.list_heads[level as usize] = unsafe { Some(&mut *node) },
        }
    }
}

unsafe impl GlobalAlloc for Locked<BuddyAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        let level = level(layout.size());
        let node = allocator.find(level);
        let ptr = (*(node as *mut ListNode)).hide(layout);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        let node = ListNode::magic(ptr, layout);
        allocator.insert(node);
    }
}
