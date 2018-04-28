extern crate rtt;
extern crate rand;

use rtt::util::vec_slist::{self, TransChecker, Expander};
use rand::Rng;

type Map<'a> = &'a [&'a [u8]];

fn main() {
    let maze: Map =
        &[b"###############",
          b"#  *   #      ##########",
          b"#      #               #",
          b"#      #      ###  #####",
          b"#      #      #        #",
          b"#             #        #",
          b"#      #      #        #",
          b"###############        #",
          b"# @    #               #",
          b"#      #      ##########",
          b"#      #      #",
          b"#      #      #",
          b"#      #      #",
          b"#             #",
          b"###############"];
    let start = find_cell(b'*', maze).unwrap();
    let finish = find_cell(b'@', maze).unwrap();
    let width = maze.iter().map(|line| line.len()).max().unwrap();
    let height = maze.len();

    let mut rng = rand::thread_rng();
    let mut iters = 0;

    let outcome = rtt::plan(
        // RandomTree
        vec_slist::RandomTree::new(ContextManager(maze)),
        // Sampler (pick a random cell on a map)
        |_: &_| {
            let row = rng.gen_range(0, height);
            let col = rng.gen_range(0, width);
            Ok::<_, ()>(Some((row, col)))
        },
        // Limiter (stop after 10000 iterations)
        |_: &_| {
            iters += 1;
            Ok::<_, ()>(iters > 10000)
        },
        // GoalChecker (check if finish is reached)
        |node: &vec_slist::RandomTreeNode<_, _>| Ok::<_, ()>(node.state() == &finish),
        // init state (start position in a maze)
        start,
    ).unwrap();

    println!("Map with {} rows and {} columns, start = {:?}, finish = {:?}", width, height, start, finish);
    match outcome {
        rtt::Outcome::PathPlanned(rev_path) => {
            let path: Vec<_> = rev_path.collect();
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

type Coord = (usize, usize);
type State = Coord;

#[derive(PartialEq, PartialOrd)]
enum Dist {
    Straight(usize),
    NotStraight,
}

struct ContextManager<'a>(Map<'a>);

impl<'a> vec_slist::ContextManager for ContextManager<'a> {
    type State = State;
    type Error = ();
    type Dist = Dist;

    fn metric_distance(&mut self, node: &Self::State, probe: &Self::State) ->
        Result<Self::Dist, Self::Error>
    {
        Ok(if let Some(path_iter) = StraightPathIter::new(node, probe) {
            Dist::Straight(path_iter.count())
        } else {
            Dist::NotStraight
        })
    }

    fn generate_trans<T>(&self, probe: Self::State, node_trans: T) ->
        Result<Option<Self::State>, Self::Error>
        where T: TransChecker<Self::State, Self::Error>
    {
        let start_coord = *node_trans.current();
        if let Some(path_iter) = StraightPathIter::new(&start_coord, &probe) {
            for coord in path_iter {
                if self.0[coord.0][coord.1] == b'#' {
                    return Ok(None);
                }
                if node_trans.already_visited(&coord, |a, b| Ok(a == b))? {
                    return Ok(None);
                }
            }
            Ok(Some(probe))
        } else {
            Ok(None)
        }
    }

    fn generate_expand<E>(&mut self, probe: Self::State, node_expander: E) ->
        Result<(), Self::Error>
        where E: Expander<Self::State, Self::Error>
    {
        let coord = *node_expander.current();
        if let Some(path_iter) = StraightPathIter::new(&coord, &probe) {
            node_expander.expand(path_iter.map(Ok))?;
        }
        Ok(())
    }
}

struct StraightPathIter<'a> {
    source: Coord,
    target: &'a Coord,
}

impl<'a> StraightPathIter<'a> {
    fn new(source: &'a Coord, target: &'a Coord) -> Option<StraightPathIter<'a>> {
        if source == target {
            None
        } else if source.0 == target.0 || source.1 == target.1 {
            Some(StraightPathIter { source: *source, target, })
        } else {
            None
        }
    }
}

impl<'a> Iterator for StraightPathIter<'a> {
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
