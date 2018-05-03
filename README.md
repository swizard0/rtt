# Rapidly-Exploring random trees path planning algorithm

## Overview

Randomized data structure that is designed for a broad class of path planning problems.

Theory:
- <http://msl.cs.uiuc.edu/~lavalle/papers/Lav98c.pdf>
- <http://msl.cs.uiuc.edu/rrt/>
- <http://planning.cs.uiuc.edu/node231.html>
- <https://stackoverflow.com/questions/11933385/rapid-exploring-random-trees>

## Library

`rtt` is a Rust crate with a very abstract algorithm implementation. Everything outside of raw algorithm (sampling, memory management, nearest node search etc) is left to library user.

Several useful data structures and helpers are available in `rtt::util` module for your convenience, such as:
- [rtt::util::rtt::vec_slist](src/util/rtt/vec_slist.rs): single-linked tree implemented over `Vec`
- [rtt::util::no_err](src/util/mod.rs): simple wrapper for `Result<T, !>` type

## Example usage

Try the [example](examples/rect_maze.rs) yourself:

```
% cargo run --example rect_maze

Maze of 15 rows and 24 cols, start: (1, 3), finish: (8, 2)
Path planned in 2707 iterations:
###############
#  ++  #  ++++##########
#   +  #  +  +++++     #
#   +  #  +   ###+ #####
#   +++#  +   #  +     #
#     +++++   #  +     #
#      #      #  +     #
###############  +     #
#++    #     +++++     #
#+     #   +++##########
#+++   #   +  #
#  +   #++++  #
#  +   #+     #
#  ++++++     #
###############
```
