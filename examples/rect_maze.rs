extern crate rtt;
extern crate rand;

use std::collections::HashSet;
use rand::Rng;

use rtt::util::rtt::vec_slist::{EmptyRandomTree, RandomTree, NodeRef};

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

    println!("Maze of {} rows and {} cols, start: {:?}, finish: {:?}", height, width, start, finish);

    let mut rng = rand::thread_rng();
    let mut visited = HashSet::new();
    let mut iters = 0;

    let rtt: EmptyRandomTree<State> =
        EmptyRandomTree::new();
    let planner = rtt::Planner::new(rtt);

    let mut planner_node = planner.add_root(|empty_rtt: EmptyRandomTree<_>| {
        let rtt = empty_rtt.add_root(start);
        let root_ref = rtt.root();
        visited.insert(start);
        RttNodeFocus::make_ok(rtt, root_ref, false)
    }).unwrap();

    loop {
        if planner_node.rtt_node().goal_reached {
            let rev_path = planner_node.into_path(|focus: RttNodeFocus| {
                Ok::<_, ()>(focus.rtt.into_path(focus.node_ref))
            }).unwrap();
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

        let mut planner_sample =
            planner_node.prepare_sample(|focus: RttNodeFocus| Ok::<_, ()>(focus.rtt)).unwrap();

        loop {
            if iters >= 10000 {
                println!("Planning limit reached");
                return;
            }

            iters += 1;
            let row = rng.gen_range(0, height);
            let col = rng.gen_range(0, width);

            let planner_pick = planner_sample.sample(|rtt| {
                Ok::<_, ()>((rtt, (row, col)))
            }).unwrap();

            let planner_closest = planner_pick.nearest_node(locate_closest).unwrap();
            if let Some(path_iter) = build_route(planner_closest.rtts_node()) {
                let blocked = path_iter.clone().any(|coord| {
                    maze[coord.0][coord.1] == b'#' || visited.contains(&coord)
                });
                if !blocked {
                    planner_node = planner_closest.transition(|rtts_node| perform_move(rtts_node, path_iter, &finish)).unwrap();
                    break;
                }
            }
            planner_sample = planner_closest.no_transition(sample_again).unwrap();
        }
    }
}

type Coord = (usize, usize);
type State = Coord;

struct RttNodeFocus {
    rtt: RandomTree<State>,
    node_ref: NodeRef,
    goal_reached: bool,
}

impl RttNodeFocus {
    fn make_ok(rtt: RandomTree<State>, node_ref: NodeRef, goal_reached: bool) -> Result<RttNodeFocus, ()> {
        Ok(RttNodeFocus { rtt, node_ref, goal_reached, })
    }
}

fn locate_closest((rtt, sample): (RandomTree<State>, State)) -> Result<(RandomTree<State>, State, NodeRef), ()> {
    fn sq_dist(coord_a: &Coord, coord_b: &Coord) -> usize {
        (((coord_a.0 as isize - coord_b.0 as isize) * (coord_a.0 as isize - coord_b.0 as isize)) +
         ((coord_a.1 as isize - coord_b.1 as isize) * (coord_a.1 as isize - coord_b.1 as isize))) as usize
    }

    let mut closest;
    {
        let states = rtt.states();
        closest = (states.root.0, sq_dist(states.root.1, &sample));
        for (node_ref, coord) in states.children {
            let dist = sq_dist(coord, &sample);
            if dist < closest.1 {
                closest = (node_ref, dist);
            }
        }
    }

    Ok((rtt, sample, closest.0))
}

fn build_route(&(ref rtt, ref dst, ref closest_node_ref): &(RandomTree<State>, State, NodeRef)) -> Option<StraightPathIter> {
    let src = rtt.get_state(closest_node_ref);
    StraightPathIter::new(src, dst)
}

fn sample_again(rtts_node: (RandomTree<State>, State, NodeRef)) -> Result<RandomTree<State>, ()> {
    Ok(rtts_node.0)
}

fn perform_move(
    (mut rtt, _sample, mut node_ref): (RandomTree<State>, State, NodeRef),
    path_iter: StraightPathIter,
    finish: &Coord
) ->
    Result<RttNodeFocus, ()>
{
    let mut goal_reached = false;
    for coord in path_iter {
        node_ref = rtt.expand(node_ref, coord);
        if &coord == finish {
            goal_reached = true;
            break;
        }
    }
    RttNodeFocus::make_ok(rtt, node_ref, goal_reached)
}

#[derive(Clone)]
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
