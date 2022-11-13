use crate::maze::Maze;
use std::hash::{Hash, Hasher};

/// Node represents each field in 2D maze, it contains `Position` and costs/weights.
///
/// It also contains heap allocation of its parent/previous `Node` that "discovered" it.
/// This is due to change, if arena allocator gets implemented.
#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) position: Position,
    pub(crate) g_cost: usize,
    pub(crate) h_cost: usize,
    pub(crate) previous: Option<Box<Node>>,
}

impl Node {
    fn new(position: Position, previous: &Node, end: Position) -> Self {
        let mut node = Node {
            position,
            g_cost: 0,
            h_cost: 0,
            previous: None,
        };

        node.h_cost = Node::heuristic(node.position, end);
        node.g_cost = Node::g_cost(position, previous);
        node
    }

    pub(crate) fn neighbours(&self, maze: &Maze) -> Vec<Node> {
        let mut neighbours = vec![];

        let offset_x = [-1, -1, 0, 1, 1, 1, 0, -1];
        let offset_y = [0, -1, -1, -1, 0, 1, 1, 1];

        let pivot_x = self.position.x();
        let pivot_y = self.position.y();

        for i in 0..8 {
            let node_x = pivot_x + offset_x[i];
            let node_y = pivot_y + offset_y[i];
            let position = Position((node_x as usize, node_y as usize));

            if Node::is_valid((node_x, node_y), maze) {
                let node = Node::new(position, self, maze.end.unwrap());
                neighbours.push(node);
            }
        }
        neighbours
    }

    pub(crate) fn lower_cost(&self, neighbour: &Node) -> bool {
        self.f_cost() < neighbour.f_cost()
            || (self.h_cost < neighbour.h_cost && self.f_cost() == neighbour.f_cost())
    }

    pub(crate) fn heuristic(position: Position, end: Position) -> usize {
        let a = (end.x() - position.x()).abs() * 10;
        let b = (end.y() - position.y()).abs() * 10;
        let c = a.pow(2) + b.pow(2);
        (c as f64).sqrt() as usize
    }

    pub(crate) fn g_cost(position: Position, prev: &Node) -> usize {
        let diagonal_positions = vec![
            (prev.position.x() - 1, prev.position.y() - 1),
            (prev.position.x() - 1, prev.position.y() + 1),
            (prev.position.x() + 1, prev.position.y() - 1),
            (prev.position.x() + 1, prev.position.y() + 1),
        ];

        if diagonal_positions.contains(&position.xy()) {
            prev.g_cost + 14
        } else {
            prev.g_cost + 10
        }
    }

    pub(crate) fn f_cost(&self) -> usize {
        self.g_cost + self.h_cost
    }

    fn is_valid(position: (isize, isize), maze: &Maze) -> bool {
        position.0 < maze.x_len() as isize
            && position.0 >= 0
            && position.1 < maze.y_len() as isize
            && position.1 >= 0
            && maze.field()[position.1 as usize][position.0 as usize] != maze.wall()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.position.0 == other.position.0
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.0.hash(state);
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
pub(crate) struct Position(pub(crate) (usize, usize));

impl Position {
    pub(crate) fn x(&self) -> isize {
        self.0 .0 as isize
    }

    pub(crate) fn y(&self) -> isize {
        self.0 .1 as isize
    }

    pub(crate) fn xy(&self) -> (isize, isize) {
        (self.0 .0 as isize, self.0 .1 as isize)
    }

    pub(crate) fn xy_usize(&self) -> (usize, usize) {
        (self.0 .0, self.0 .1)
    }
}

/// Wrapper around `f_cost` that represents priority inside the `PriorityQueue`.
///
/// It has custom implementation of `PartialOrd` and `Ord` traits to provide correct functionality when getting
/// popped out of a priority queue.
pub(crate) struct Priority(pub(crate) usize);

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Priority {}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}
