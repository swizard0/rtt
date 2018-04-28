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
    let width = maze[0].len();
    let height = maze.len();

    let mut rng = rand::thread_rng();
    let mut iters = 0;

    let rtt = RandomTree::new(maze);
    let outcome = rtt::plan(
        rtt,
        |_: &_| {
            let row = rng.gen_range(0, height);
            let col = rng.gen_range(0, width);
            Ok::<_, ()>(Some(State { target: (row, col), }))
        },
        |_: &_| {
            iters += 1;
            Ok::<_, ()>(iters > 10000)
        },
        |node: &RandomTreeNode<'_>| Ok::<_, ()>(node.coord() == finish),
        State { target: start, },
    ).unwrap();

    println!("Map {}x{}, start = {:?}, finish = {:?}", width, height, start, finish);
    match outcome {
        rtt::Outcome::PathPlanned(path) => {
            println!("Path planned:");
            for item in cells_iter(maze) {
                match item {
                    MapItem::NextRow =>
                        println!(""),
                    MapItem::Cell { coord, tile, } =>
                        print!("{}", if path.contains(&coord) { '+' } else { tile as char }),
                }
            }
        },
        rtt::Outcome::NoPathExists =>
            println!("No path exists"),
        rtt::Outcome::LimitReached =>
            println!("Planning limit reached"),
    }
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

    fn make_root(mut self, state: Self::State) -> Result<Self::Node, Self::Error> {
        self.0.nodes.clear();
        self.0.nodes.push(PathNode { coord: state.target, prev: None, });
        Ok(RandomTreeNode {
            tree: self.0,
            node: 0,
        })
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
            node: nearest.map(|m| m.1).unwrap_or(0),
        })
    }
}

struct RandomTreeNode<'a> {
    tree: Tree<'a>,
    node: usize,
}

impl<'a> RandomTreeNode<'a> {
    fn coord(&self) -> Coord {
        self.tree.nodes[self.node].coord
    }
}

impl<'a> rtt::RandomTreeNode for RandomTreeNode<'a> {
    type State = State;
    type Error = ();
    type Tree = RandomTree<'a>;
    type Path = Vec<Coord>;

    fn expand(mut self, state: Self::State) -> Result<Self, Self::Error> {
        let mut node_index = self.node;
        if let Some(path_iter) = PathLineIter::new(self.tree.nodes[node_index].coord, state.target) {
            for coord in path_iter {
                let next_index = self.tree.nodes.len();
                self.tree.nodes.push(PathNode { coord, prev: Some(node_index), });
                node_index = next_index;
            }
        }
        Ok(RandomTreeNode { tree: self.tree, node: node_index, })
    }

    fn transition(&self, random_state: Self::State) -> Result<Option<Self::State>, Self::Error> {
        if let Some(path_iter) = PathLineIter::new(self.coord(), random_state.target) {
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
        let mut maybe_index = Some(self.node);
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
        if source == target {
            None
        } else if source.0 == target.0 || source.1 == target.1 {
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

enum MapItem {
    Cell { coord: Coord, tile: u8, },
    NextRow,
}

fn cells_iter<'a>(map: Map<'a>) -> Box<Iterator<Item = MapItem> + 'a> {
    let iter = map.iter()
        .enumerate()
        .flat_map(|(row, &line)| {
            line.iter()
                .enumerate()
                .map(move |(col, &tile)| MapItem::Cell { coord: (row, col), tile, })
                .chain(Some(MapItem::NextRow).into_iter())
        });
    Box::new(iter)
}

fn find_cell<'a>(cell: u8, map: Map<'a>) -> Option<Coord> {
    cells_iter(map)
        .filter_map(|item| match item {
            MapItem::Cell { tile, coord } if tile == cell => Some(coord),
            _ => None,
        })
        .next()
}
