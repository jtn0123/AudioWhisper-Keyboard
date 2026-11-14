//! Android fetches nodes; Rust owns traversal order, pruning, text selection and scroller policy.
use crate::screen::{self, Bounds};
use chars::is_whitespace;
pub const MAX_NODES: usize = 20_000;
pub const MAX_DEPTH: usize = 250;
pub const TURN_MS: i64 = 200;
pub const MAX_TURNS: i32 = 8;
#[derive(Debug, Clone)]
pub struct Node {
    pub bounds: Bounds,
    pub password: bool,
    pub editable: bool,
    pub visible: bool,
    pub scrollable: bool,
    pub rows: i32,
    pub columns: i32,
    pub up_down: bool,
    pub across: bool,
    pub class: Vec<u16>,
    pub children: i32,
}
pub trait Tree {
    type Error;
    fn node(&mut self, id: i32) -> Result<Node, Self::Error>;
    fn text(&mut self, id: i32) -> Result<Vec<u16>, Self::Error>;
    fn child(&mut self, id: i32, index: i32) -> Result<Option<i32>, Self::Error>;
}
#[derive(Debug)]
pub struct Piece {
    pub id: i32,
    pub bounds: Bounds,
    pub scroller: Option<usize>,
}
#[derive(Debug)]
struct Scroller {
    id: i32,
    parent: Option<usize>,
    area: i64,
}
#[derive(Debug)]
pub struct Walk {
    pub pieces: Vec<Piece>,
    scrollers: Vec<Scroller>,
    pub nodes: usize,
    pub web: bool,
    visible: Bounds,
}
impl Walk {
    pub fn run<T: Tree>(tree: &mut T, visible: Bounds) -> Result<Self, T::Error> {
        let mut walk = Self {
            pieces: vec![],
            scrollers: vec![],
            nodes: 0,
            web: false,
            visible,
        };
        walk.visit(tree, 0, None, false, 0)?;
        Ok(walk)
    }
    fn visit<T: Tree>(
        &mut self,
        tree: &mut T,
        id: i32,
        scroller: Option<usize>,
        sideways: bool,
        depth: usize,
    ) -> Result<(), T::Error> {
        if self.nodes >= MAX_NODES || depth > MAX_DEPTH {
            return Ok(());
        }
        self.nodes += 1;
        let node = tree.node(id)?;
        if node.password || node.editable {
            return Ok(());
        }
        if sideways && !node.visible {
            return Ok(());
        }
        self.web |= node.class == "android.webkit.WebView".encode_utf16().collect::<Vec<_>>();
        let mut inner = scroller;
        let mut across = sideways;
        if node.scrollable {
            if screen::is_sideways(
                node.rows,
                node.columns,
                node.up_down,
                node.across,
                &node.class,
            ) {
                across = true;
            } else {
                self.scrollers.push(Scroller {
                    id,
                    parent: scroller,
                    area: node.bounds.visible_area(self.visible),
                });
                inner = Some(self.scrollers.len() - 1);
            }
        }
        let before = self.pieces.len();
        for index in 0..node.children {
            if let Some(child) = tree.child(id, index)? {
                self.visit(tree, child, inner, across, depth + 1)?;
            }
        }
        if self.pieces.len() == before {
            let text = tree.text(id)?;
            if text.iter().any(|c| !is_whitespace(*c)) {
                self.pieces.push(Piece {
                    id,
                    bounds: node.bounds,
                    scroller,
                });
            }
        }
        Ok(())
    }
    /// main node id, visited, web; then each selected node, top, bottom, in-main-list.
    pub fn packet(&self) -> Vec<i32> {
        let parents = self.scrollers.iter().map(|s| s.parent).collect::<Vec<_>>();
        let main = screen::main_scroller(
            &self.pieces.iter().map(|p| p.scroller).collect::<Vec<_>>(),
            &self.scrollers.iter().map(|s| s.area).collect::<Vec<_>>(),
        );
        let mut packet = vec![
            main.map_or(-1, |i| self.scrollers[i].id),
            self.nodes as i32,
            i32::from(self.web),
        ];
        for p in &self.pieces {
            packet.extend([
                p.id,
                p.bounds.top,
                p.bounds.bottom,
                i32::from(screen::is_within(&parents, p.scroller, main)),
            ]);
        }
        packet
    }
}
