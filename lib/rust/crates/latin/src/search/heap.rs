//! The binary heap `std::priority_queue` is, as libc++'s `std::push_heap` and `std::pop_heap`
//! build it (`__sift_up`, `__floyd_sift_down`). The search's comparisons are not a strict weak
//! ordering (two nodes within `MIN_DIFF` of each other compare as equal in one step and unequal
//! in the next), so which of several near-equal nodes comes out first depends on how the heap
//! moves them. Porting the same moves keeps every pop in the C++'s order.
//!
//! `less(a, b)` is the C++ comparator: `true` when `a` goes below `b`. The greatest element is at
//! index 0. Elements move by swaps, which leaves the same array as libc++'s holes and moves do.

/// `std::push_heap` on `heap`, whose last element is the one just pushed (`__sift_up`).
pub fn push_heap<T>(heap: &mut [T], less: impl Fn(&T, &T) -> bool) {
    sift_up(heap, heap.len(), &less);
}

/// `std::pop_heap`: moves the greatest element to the end of `heap`, leaving a heap in the rest.
pub fn pop_heap<T>(heap: &mut [T], less: impl Fn(&T, &T) -> bool) {
    let len = heap.len();
    if len <= 1 {
        return;
    }
    let hole = floyd_sift_down(heap, len, &less);
    let last = len - 1;
    if hole != last {
        // *hole = move(*last); ++hole; *last = move(top): with the top parked at `last`, a swap
        // does both, and the element moved into the hole is then sifted up.
        heap.swap(hole, last);
        sift_up(heap, hole + 1, &less);
    }
}

/// `__sift_up(first, last, comp, len)`: moves the element at `len - 1` up while its parent is less.
fn sift_up<T>(heap: &mut [T], len: usize, less: &impl Fn(&T, &T) -> bool) {
    if len <= 1 {
        return;
    }
    let mut child = len - 1;
    let mut parent = (len - 2) / 2;
    if !less(&heap[parent], &heap[child]) {
        return;
    }
    loop {
        heap.swap(parent, child);
        child = parent;
        if child == 0 {
            break;
        }
        parent = (child - 1) / 2;
        if !less(&heap[parent], &heap[child]) {
            break;
        }
    }
}

/// `__floyd_sift_down`: the hole at the root goes down to a leaf, taking the greater child each
/// time. The top is parked at the hole as it goes, and the caller moves it to the end.
fn floyd_sift_down<T>(heap: &mut [T], len: usize, less: &impl Fn(&T, &T) -> bool) -> usize {
    let mut hole = 0;
    let mut child_i = 0;
    let mut child = 0;
    loop {
        child_i += child + 1;
        child = 2 * child + 1;
        if child + 1 < len && less(&heap[child_i], &heap[child_i + 1]) {
            child_i += 1;
            child += 1;
        }
        // *hole = move(*child_i): the child moves up and the parked top follows the hole down.
        heap.swap(hole, child_i);
        hole = child_i;
        if child > (len - 2) / 2 {
            return hole;
        }
    }
}
