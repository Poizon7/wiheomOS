use super::Locked;
use core::alloc::{GlobalAlloc, Layout};

#[derive(Clone, Copy)]
enum Flag {
    Free,
    Taken,
}

struct ListNode {
    flag: Flag,
    level: u8,
    next: Option<&'static mut ListNode>,
    prev: Option<&'static mut ListNode>,
}

const MIN: u8 = 5;
const LEVELS: u8 = 8;
// const PAGE: u32 = 4096;

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
    pub unsafe fn init(&mut self, heap_start: usize, _heap_size: usize) {
        // println!("mem start at {:x}", heap_start);
        let ptr = heap_start as *mut ListNode;
        unsafe {
            (*ptr).level = LEVELS - 1;
            (*ptr).flag = Flag::Free;

            self.list_heads[self.list_heads.len() - 1] = Some(&mut *ptr)
        }
    }
}

impl ListNode {
    fn buddy(&mut self) -> *mut Self {
        let index = self.level;
        let mask = 0x1 << (index + MIN);
        (self as *mut ListNode as i32 ^ mask) as *mut ListNode
    }

    fn split(&mut self) -> *mut Self {
        let index = self.level - 1;
        let mask = 0x1 << (index + MIN);
        (self as *mut ListNode as usize | mask) as *mut ListNode
    }

    fn primary(&mut self) -> *mut Self {
        let index = self.level - 1;
        // let mask = 0xffffffffffffffff << (1 + index + MIN);
        let mask = -1 << (1 + index + MIN);
        (self as *mut ListNode as i32 & mask) as *mut ListNode
    }

    fn hide(&mut self) -> *mut u8 {
        (self as *mut ListNode as *mut u8).wrapping_add(1)
    }

    fn magic(mem: *mut u8) -> *mut Self {
        mem.wrapping_sub(1) as *mut ListNode
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
        // println!("try and find mem at level {}", level);
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
                    // println!("checking {}", current);
                    if let Some(node) = self.list_heads[current].take() {
                        self.list_heads[current] = Some(node);
                        found_level += 1;
                        break;
                    }

                    found_level += 1;
                }

                for current in ((level + 1)..found_level).rev() {
                    // println!("spliting at {}", current);
                    match self.list_heads[current].take() {
                        Some(node) => {
                            // println!("node at {:p}", node);
                            let buddy = (*node).split();
                            // println!("buddy at {:p}", node);

                            node.level = (current - 1) as u8;

                            // println!("node level updated");

                            unsafe { (*buddy).level = (current - 1) as u8 };

                            // println!("buddy level updated");

                            node.next = unsafe { Some(&mut *buddy) };
                            // println!("buddy added to list");
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
        unsafe { (*(node as *mut ListNode)).hide() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();
        let node = ListNode::magic(ptr);
        allocator.insert(node);
    }
}
