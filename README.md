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

## Example usage

Try the [example](https://github.com/swizard0/rtt/blob/master/examples/rect_maze.rs) yourself:

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
