extern crate rtt;
extern crate rand;

use std::collections::HashSet;
use rand::Rng;

use rtt::util::rtt_impl::vec_slist::{self, NearestNodeLocator};

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
    // let mut visited = HashSet::new();

    let rtt = vec_slist::RandomTree::new(Locator);
    let planner = rtt::Planner::new(rtt);
    let mut planner_node = planner.init(start).unwrap();
    for iters in 0 .. 10000 {
        if planner_node.rtt_node().state() == &finish {
            let rev_path = planner_node.into_path();
            let path: Vec<_> = rev_path.collect();
            println!("Path planned in {} iterations:", iters);
            for item in cells_iter(maze) {
                match item {
                    MapItem::NextRow =>
                        println!(""),
                    MapItem::Cell { coord, tile, } =>
                        print!("{}", if path.contains(&coord) { '+' } else { tile as char }),
                }
            }
            return;
        }

        let mut planner_sample = planner_node.prepare_sample();
        loop {
            let row = rng.gen_range(0, height);
            let col = rng.gen_range(0, width);

            let planner_nearest = planner_sample.sample((row, col)).unwrap();
            if let Some(path_iter) = StraightPathIter::new(planner_nearest.rtt_node().state(), planner_nearest.sample_state()) {
                let mut planner_expand = planner_nearest.start_transition();
                for coord in path_iter {
                    planner_expand = planner_expand.intermediate_trans(coord).unwrap();
                }
                planner_node = planner_expand.finish().unwrap();
                break;
            } else {
                planner_sample = planner_nearest.no_transition();
            }
        }
    }

    println!("Planning limit reached");
}

type Coord = (usize, usize);
type State = Coord;

#[derive(PartialEq, PartialOrd)]
enum Dist {
    Straight(usize),
    NotStraight,
}

struct Locator;

impl vec_slist::NearestNodeLocator<State> for Locator {
    type Error = ();

    fn locate_nearest<'a, SI, I>(&mut self, state: &State, rtt_root: SI, rtt_states: I) ->
        Result<&'a SI, Self::Error>
        where I: Iterator<Item = SI>, SI: AsRef<State>
    {

        unimplemented!()
    }
}

// struct ContextManager<'a>(Map<'a>);

// impl<'a> vec_slist::ContextManager for ContextManager<'a> {
//     type State = State;
//     type Error = ();
//     type Dist = Dist;

//     fn metric_distance(&mut self, node: &Self::State, probe: &Self::State) ->
//         Result<Self::Dist, Self::Error>
//     {
//         Ok(if let Some(path_iter) = StraightPathIter::new(node, probe) {
//             Dist::Straight(path_iter.count())
//         } else {
//             Dist::NotStraight
//         })
//     }

//     fn generate_trans<T, E>(&self, probe: Self::State, node_trans: T) ->
//         Result<Option<Self::State>, CMError<Self::Error, E>>
//         where T: TransChecker<Self::State, E>
//     {
//         let start_coord = *node_trans.current();
//         if let Some(path_iter) = StraightPathIter::new(&start_coord, &probe) {
//             for coord in path_iter {
//                 if self.0[coord.0][coord.1] == b'#' {
//                     return Ok(None);
//                 }
//                 if node_trans.already_visited(&coord).map_err(CMError::RandomTreeProc)? {
//                     return Ok(None);
//                 }
//             }
//             Ok(Some(probe))
//         } else {
//             Ok(None)
//         }
//     }

//     fn generate_expand<E, EE>(&mut self, probe: Self::State, node_expander: E) ->
//         Result<(), CMError<Self::Error, EE>>
//         where E: Expander<Self::State, EE>
//     {
//         let coord = *node_expander.current();
//         if let Some(path_iter) = StraightPathIter::new(&coord, &probe) {
//             node_expander.expand(path_iter.map(Ok))
//                 .map_err(CMError::RandomTreeProc)?;
//         }
//         Ok(())
//     }
// }

struct StraightPathIter {
    source: Coord,
    target: Coord,
}

impl StraightPathIter {
    fn new(source: &Coord, target: &Coord) -> Option<StraightPathIter> {
        if source == target {
            None
        } else if source.0 == target.0 || source.1 == target.1 {
            Some(StraightPathIter {
                source: source.clone(),
                target: target.clone(),
            })
        } else {
            None
        }
    }
}

impl Iterator for StraightPathIter {
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
