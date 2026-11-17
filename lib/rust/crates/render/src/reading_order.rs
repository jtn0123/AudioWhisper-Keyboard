//! The order a screen reader's swipes visit the keyboard: the order a sighted reader scans it.
//!
//! The strip at the top comes first (the Smartbar's buttons and words, the typed-text preview, or
//! the search strip), then the keys, or the open panel. Each part reads row by row, top to bottom,
//! and each row left to right as drawn. A popup reads as one block straight after the key or item
//! that opened it, so one swipe reaches its choices; with nothing on screen that opened it, it
//! reads first in the body. A node that holds another's centre (a scrolling area, a backdrop)
//! reads just before the first thing inside it.
//!
//! The list order was also touch's stacking order, so every node keeps its draw layer, and names
//! the node read before it so Android can be told the order outright.
use crate::accessibility::Node;
use keys::Rect;

/// Where a node belongs in the reading order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// The strip above the keys.
    Top,
    /// The keys, or the open panel.
    Body,
    /// A popup: straight after the node with this id, or first in the body without one.
    After(Option<i32>),
}

/// Takes nodes in draw order (later on top), and returns them in reading order.
pub fn arrange(placed: Vec<(Place, Node)>) -> Vec<Node> {
    let mut top = Vec::new();
    let mut body = Vec::new();
    let mut popups: Vec<(Option<i32>, Vec<Node>)> = Vec::new();
    for (layer, (place, mut node)) in placed.into_iter().enumerate() {
        node.layer = layer;
        match place {
            Place::Top => top.push(node),
            Place::Body => body.push(node),
            Place::After(source) => match popups.iter_mut().find(|p| p.0 == source) {
                Some(p) => p.1.push(node),
                None => popups.push((source, vec![node])),
            },
        }
    }
    let mut out = rows(top);
    let mut body = rows(body);
    let mut first = 0;
    for (source, nodes) in popups {
        let block = rows(nodes);
        let at = match source.and_then(|id| body.iter().position(|n| n.id == id)) {
            Some(i) => i + 1,
            None => {
                let at = first;
                first += block.len();
                at
            }
        };
        body.splice(at..at, block);
    }
    out.extend(body);
    let mut before = None;
    for node in &mut out {
        node.after = before;
        before = Some(node.id);
    }
    out
}

/// Rows top to bottom, each left to right. A node joins a row while it starts above the middle of
/// the row's first node, so rows of unequal heights and slightly offset tops still read as one.
pub fn rows(nodes: Vec<Node>) -> Vec<Node> {
    let n = nodes.len();
    let b = |i: usize| nodes[i].bounds;
    let holds = |outer: usize, inner: usize| {
        let (o, i) = (b(outer), b(inner));
        let (x, y) = ((i.left + i.right) / 2.0, (i.top + i.bottom) / 2.0);
        area(i) < area(o) && x >= o.left && x < o.right && y >= o.top && y < o.bottom
    };
    let holds_any: Vec<bool> = (0..n).map(|i| (0..n).any(|j| holds(i, j))).collect();
    // A wrapper holds a plain node; one that holds only other wrappers reads like a plain node.
    let wrapper: Vec<bool> = (0..n)
        .map(|i| holds_any[i] && (0..n).any(|j| !holds_any[j] && holds(i, j)))
        .collect();
    let mut plain: Vec<usize> = (0..n).filter(|&i| !wrapper[i]).collect();
    plain.sort_by(|&x, &y| (b(x).top.total_cmp(&b(y).top)).then(b(x).left.total_cmp(&b(y).left)));
    let mut order = Vec::with_capacity(n);
    let mut row: Vec<usize> = Vec::new();
    for i in plain {
        if let Some(&first) = row.first()
            && b(i).top >= (b(first).top + b(first).bottom) / 2.0
        {
            flush(&mut row, &mut order, b);
        }
        row.push(i);
    }
    flush(&mut row, &mut order, b);
    let mut wrappers: Vec<usize> = (0..n).filter(|&i| wrapper[i]).collect();
    wrappers.sort_by(|&x, &y| area(b(y)).total_cmp(&area(b(x))));
    for w in wrappers {
        let at = order
            .iter()
            .position(|&j| !wrapper[j] && holds(w, j))
            .unwrap_or(order.len());
        order.insert(at, w);
    }
    let mut slots: Vec<Option<Node>> = nodes.into_iter().map(Some).collect();
    order.into_iter().filter_map(|i| slots[i].take()).collect()
}

fn flush(row: &mut Vec<usize>, order: &mut Vec<usize>, b: impl Fn(usize) -> Rect) {
    row.sort_by(|&x, &y| (b(x).left.total_cmp(&b(y).left)).then(b(x).top.total_cmp(&b(y).top)));
    order.append(row);
}

fn area(r: Rect) -> f32 {
    r.width().max(0.0) * r.height().max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::Target;

    fn node(id: i32, left: f32, top: f32, right: f32, bottom: f32) -> Node {
        let bounds = Rect {
            left,
            top,
            right,
            bottom,
        };
        Node::new(id, bounds, String::new(), "key", Target::Key(0))
    }
    fn ids(nodes: &[Node]) -> Vec<i32> {
        nodes.iter().map(|n| n.id).collect()
    }

    #[test]
    fn offset_tops_stay_in_one_row_and_a_nested_backdrop_reads_first() {
        let nodes = vec![
            node(1, 50.0, 4.0, 100.0, 40.0),
            node(2, 0.0, 0.0, 50.0, 40.0),
            node(3, 0.0, 40.0, 100.0, 80.0),
            node(4, 0.0, 0.0, 100.0, 80.0),
            node(5, 0.0, 0.0, 120.0, 90.0),
        ];
        assert_eq!(ids(&rows(nodes)), vec![5, 4, 2, 1, 3]);
    }

    #[test]
    fn a_popup_with_no_opener_on_screen_reads_first_in_the_body() {
        let placed = vec![
            (Place::Body, node(1, 0.0, 40.0, 50.0, 80.0)),
            (Place::Top, node(2, 0.0, 0.0, 50.0, 40.0)),
            (Place::After(Some(99)), node(3, 50.0, 40.0, 90.0, 60.0)),
            (Place::After(None), node(4, 0.0, 60.0, 40.0, 80.0)),
        ];
        let out = arrange(placed);
        assert_eq!(ids(&out), vec![2, 3, 4, 1]);
        assert_eq!(
            out.iter().map(|n| n.layer).collect::<Vec<_>>(),
            [1, 2, 3, 0]
        );
        assert_eq!(
            out.iter().map(|n| n.after).collect::<Vec<_>>(),
            [None, Some(2), Some(3), Some(4)]
        );
    }
}
