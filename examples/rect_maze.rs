extern crate rtt;
extern crate rand;

use std::cmp::{min, max};
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
    let width = maze.iter().map(|line| line.len()).max().unwrap();
    let height = maze.len();
    let start = find_cell(b'*', maze, height, width).unwrap();
    let finish = find_cell(b'@', maze, height, width).unwrap();

    println!("Maze of {} rows and {} cols, start: {:?}, finish: {:?}", height, width, start, finish);

    let mut rng = rand::thread_rng();
    let mut visited = HashSet::new();
    let mut iters = 0;
    let rev_path;

    let planner = rtt::PlannerInit::new(EmptyRandomTree::new());
    let planner = planner.add_root_ok(|empty_rtt| Ok(empty_rtt.add_root(start)));
    let mut planner_node = planner.root_node_ok(|rtt: &mut _| {
        let root_ref = rtt.root();
        visited.insert(start);
        Ok(RttNodeFocus { node_ref: root_ref, goal_reached: false, })
    });

    loop {
        if planner_node.rtt_node().goal_reached {
            rev_path = planner_node.into_path_ok(|focus| Ok(focus.rtt.into_path(focus.node_ref)));
            break;
        }
        let mut planner_sample = planner_node.prepare_sample_ok(|_rtt: &mut _, _focus| Ok(()));

        loop {
            if iters >= 10000 {
                println!("Planning limit reached");
                return;
            }
            iters += 1;

            let planner_sample = planner_ready_to_sample.sample_ok(|rtt: &mut _| {
                Ok((rng.gen_range(0, height), rng.gen_range(0, width)))
            });
            let planner_closest =
                planner_sample.closest_to_sample_ok(|rtt: &mut _, sample| locate_closest(v, &sample)); // ?????

            let route = {
                let &RttTrans { ref rtt, sample: ref dst, ref closest_node_ref, } = planner_closest.rtts_node();
                StraightPathIter::new(rtt.get_state(closest_node_ref), dst)
            };
            if let Some(path_iter) = route {
                let blocked = path_iter.clone().any(|coord| {
                    maze[coord.0][coord.1] == b'#' || visited.contains(&coord)
                });
                if !blocked {
                    planner_node =
                        planner_closest.transition(|rtts_node| perform_move(rtts_node, path_iter, &finish, &mut visited)).unwrap();
                    break;
                }
            }
            planner_sample = planner_closest.no_transition(|trans: RttTrans| no_err(trans.rtt)).unwrap();
        }
    }

    let path: Vec<_> = rev_path.collect();
    println!("Path planned in {} iterations:", iters);
    for row in 0 .. height {
        for col in 0 .. min(width, maze[row].len()) {
            print!("{}", if path.contains(&(row, col)) { '+' } else { maze[row][col] as char });
        }
        println!("");
    }
}

type Coord = (usize, usize);

struct RttNodeFocus {
    node_ref: NodeRef,
    goal_reached: bool,
}

struct RttTrans {
    rtt: RandomTree<Coord>,
    sample: Coord,
    closest_node_ref: NodeRef,
}

fn locate_closest(rtt: RandomTree<Coord>, sample: &Coord) -> Result<RttTrans, !> {
    fn manhattan(coord_a: &Coord, coord_b: &Coord) -> usize {
        (max(coord_a.0, coord_b.0) - min(coord_a.0, coord_b.0)) +
            (max(coord_a.1, coord_b.1) - min(coord_a.1, coord_b.1))
    }
    let mut closest;
    {
        let states = rtt.states();
        closest = (states.root.0, manhattan(states.root.1, sample));
        for (node_ref, coord) in states.children {
            let dist = manhattan(coord, sample);
            if dist < closest.1 {
                closest = (node_ref, dist);
            }
        }
    }
    no_err(RttTrans { rtt, sample: *sample, closest_node_ref: closest.0, })
}

fn perform_move(
    RttTrans { mut rtt, closest_node_ref: mut node_ref, .. }: RttTrans,
    path_iter: StraightPathIter,
    finish: &Coord,
    visited: &mut HashSet<Coord>,
) ->
    Result<RttNodeFocus, !>
{
    let mut goal_reached = false;
    for coord in path_iter {
        node_ref = rtt.expand(node_ref, coord);
        if &coord == finish {
            goal_reached = true;
            break;
        }
        visited.insert(coord);
    }
    no_err(RttNodeFocus { rtt, node_ref, goal_reached, })
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

fn find_cell<'a>(cell: u8, map: Map<'a>, height: usize, width: usize) -> Option<Coord> {
    for row in 0 .. height {
        for col in 0 .. min(width, map[row].len()) {
            if map[row][col] == cell {
                return Some((row, col));
            }
        }
    }
    None
}
