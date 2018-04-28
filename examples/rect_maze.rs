extern crate rtt;
extern crate rand;

use rand::Rng;

fn main() {
    let maze: Map =
        &[b"###############",
          b"#  *   #      #",
          b"#      #      #",
          b"#      #      #",
          b"#      #      #",
          b"#             #",
          b"#      #      #",
          b"############ ##",
          b"# @    #      #",
          b"#      #      #",
          b"#      #      #",
          b"#      #      #",
          b"#      #      #",
          b"#             #",
          b"###############"];
    let start = find_cell(b'*', maze).unwrap();
    let finish = find_cell(b'@', maze).unwrap();
    let rtt = RandomTree::new(maze);

    rtt::plan(
        rtt,
        |r: &_| Ok::<_, ()>(Some(State { target: (1, 4), })),
        |r: &_| Ok::<_, ()>(false),
        |n: &_| Ok::<_, ()>(true),
        State { target: start, },
    ).unwrap();
}

type Map<'a> = &'a [&'a [u8]];
type Coord = (usize, usize);

struct State {
    target: Coord,
}

struct PathNode {
    coord: Coord,
    prev: Option<usize>,
}

struct Tree<'a> {
    nodes: Vec<PathNode>,
    map: Map<'a>,
}

struct RandomTree<'a>(Tree<'a>);

impl<'a> RandomTree<'a> {
    fn new(map: Map<'a>) -> RandomTree<'a> {
        RandomTree(Tree { nodes: Vec::new(), map, })
    }
}

impl<'a> rtt::RandomTree for RandomTree<'a> {
    type State = State;
    type Error = ();
    type Node = RandomTreeNode<'a>;

    fn root(self) -> Self::Node {
        RandomTreeNode {
            tree: self.0,
            node: None,
        }
    }

    fn nearest_node(self, state: &Self::State) -> Result<Self::Node, Self::Error> {
        let mut nearest: Option<(usize, usize)> = None;
        for (index, &PathNode { coord, .. }) in self.0.nodes.iter().enumerate() {
            if let Some(path_iter) = PathLineIter::new(coord, state.target) {
                let dist = path_iter.count();
                if nearest.as_ref().map(|m| dist < m.0).unwrap_or(true) {
                    nearest = Some((dist, index));
                }
            }
        }
        Ok(RandomTreeNode {
            tree: self.0,
            node: nearest.map(|m| m.1),
        })
    }
}

struct RandomTreeNode<'a> {
    tree: Tree<'a>,
    node: Option<usize>,
}

impl<'a> rtt::RandomTreeNode for RandomTreeNode<'a> {
    type State = State;
    type Error = ();
    type Tree = RandomTree<'a>;
    type Path = Vec<Coord>;

    fn expand(mut self, state: Self::State) -> Result<Self, Self::Error> {
        let node =
            if let Some(mut node_index) = self.node {
                if let Some(path_iter) = PathLineIter::new(self.tree.nodes[node_index].coord, state.target) {
                    for coord in path_iter {
                        let next_index = self.tree.nodes.len();
                        self.tree.nodes.push(PathNode { coord, prev: Some(node_index), });
                        node_index = next_index;
                    }
                }
                Some(node_index)
            } else {
                let node_index = self.tree.nodes.len();
                self.tree.nodes.push(PathNode { coord: state.target, prev: None, });
                Some(node_index)
            };
        Ok(RandomTreeNode { tree: self.tree, node, })
    }

    fn transition(&self, random_state: Self::State) -> Result<Option<Self::State>, Self::Error> {
        let source = self.node.map(|index| self.tree.nodes[index].coord).unwrap_or(random_state.target);
        if let Some(path_iter) = PathLineIter::new(source, random_state.target) {
            for coord in path_iter {
                if self.tree.map[coord.0][coord.1] == b'#' {
                    return Ok(None);
                }
                if self.tree.nodes.iter().any(|n| n.coord == coord) {
                    return Ok(None);
                }
            }
            Ok(Some(random_state))
        } else {
            Ok(None)
        }
    }

    fn into_tree(self) -> Self::Tree {
        RandomTree(self.tree)
    }

    fn into_path(self) -> Self::Path {
        let mut path = Vec::new();
        let mut maybe_index = self.node;
        while let Some(node_index) = maybe_index {
            let node = &self.tree.nodes[node_index];
            path.push(node.coord);
            maybe_index = node.prev;
        }
        path.reverse();
        path
    }
}

struct PathLineIter {
    source: Coord,
    target: Coord,
}

impl PathLineIter {
    fn new(source: Coord, target: Coord) -> Option<PathLineIter> {
        if source.0 == target.0 || source.1 == target.1 {
            Some(PathLineIter { source, target, })
        } else {
            None
        }
    }
}

impl Iterator for PathLineIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        self.source = if self.target.0 < self.source.0 {
            (self.source.0 - 1, self.source.1)
        } else if self.target.0 > self.source.0 {
            (self.source.0 + 1, self.source.1)
        } else if self.target.1 < self.source.1 {
            (self.source.0, self.source.1 - 1)
        } else if self.target.1 > self.source.1 {
            (self.source.0, self.source.1 + 1)
        } else {
            return None;
        };
        Some(self.source)
    }
}

fn find_cell<'a>(cell: u8, map: Map<'a>) -> Option<Coord> {
    for (row, &line) in map.iter().enumerate() {
        for (col, &tile) in line.iter().enumerate() {
            if tile == cell {
                return Some((row, col));
            }
        }
    }
    None
}
