// Heuristic cost = Euclidean Distance
// A* Pathfinder Algorithm

use core::cell::RefCell;
use std::rc::Rc;

type NodeVector = Vec<WrappedNode>;
type WrappedNode = Rc<RefCell<Node>>;

#[derive(PartialEq, Clone, Debug)]
struct Node {
    position: (usize, usize),
    g_cost: usize,
    h_cost: usize,
    f_cost: usize,
    parent: Option<WrappedNode>,
}
impl Node {
    fn new(position: (usize, usize), (ey, ex): (usize, usize)) -> Node {
        let h_cost =
            ((((ex - position.1) * 10) * 2 + ((ey - position.0) * 10) * 2) as f64).sqrt() as usize;
        Node {
            position,
            g_cost: 0,
            h_cost,
            f_cost: h_cost,
            parent: None,
        }
    }
    fn neighbours(&self, field: &Vec<Vec<u8>>) -> NodeVector {
        let (ey, ex) = get_end_field(field);
        let offset_x: Vec<i32> = vec![-1, -1, 0, 1, 1, 1, 0, -1];
        let offset_y: Vec<i32> = vec![0, -1, -1, -1, 0, 1, 1, 1];
        let mut neighbours: NodeVector = vec![];
        for num in 0..8 {
            let nx = self.position.1 as i32 + offset_x[num];
            let ny = self.position.0 as i32 + offset_y[num];
            if nx < 0
                || nx > field[0].len() as i32 - 1
                || ny < 0
                || ny > field.len() as i32 - 1
                || field[ny as usize][nx as usize] == b'W'
            {
                continue;
            }
            let node = Rc::new(RefCell::new(Node::new(
                (ny as usize, nx as usize),
                (ey, ex),
            )));
            node.borrow_mut().parent = Some(Rc::new(RefCell::new(self.clone())));
            neighbours.push(node);
        }
        neighbours
    }
    fn calculate_cost(&mut self, pivot_node: &WrappedNode, end_node: &Node) {
        let (y, x) = (self.position.0 as i32, self.position.1 as i32);
        let (py, px) = (
            pivot_node.borrow().position.0 as i32,
            pivot_node.borrow().position.1 as i32,
        );
        let diagonals: Vec<(i32, i32)> = vec![
            (y + 1, x + 1),
            (y + 1, x - 1),
            (y - 1, x - 1),
            (y - 1, x + 1),
        ];
        if diagonals.contains(&(py, px)) {
            self.g_cost = pivot_node.borrow().g_cost + 14;
            self.h_cost = ((((end_node.position.1 - x as usize) * 10).pow(2)
                + ((end_node.position.0 - y as usize) * 10).pow(2)) as f64)
                .sqrt() as usize;
            self.f_cost = self.g_cost + self.h_cost;
        } else {
            self.g_cost = pivot_node.borrow().g_cost + 10;
            self.h_cost = ((((end_node.position.1 - x as usize) * 10) * 2
                + ((end_node.position.0 - y as usize) * 10) * 2) as f64)
                .sqrt() as usize;
            self.f_cost = self.g_cost + self.h_cost;
        }
    }
}

fn min_price_node(open: &NodeVector) -> WrappedNode {
    let mut min = open[0].borrow().f_cost;
    for node in open {
        if node.borrow().f_cost < min {
            min = node.borrow().f_cost
        }
    }
    for node in open.iter() {
        if node.borrow().f_cost == min {
            return Rc::clone(node);
        }
    }
    unreachable!("Open vector must contain a node!");
}
fn contains_node(vector: &NodeVector, node: &WrappedNode) -> Option<WrappedNode> {
    for vec_node in vector {
        if vec_node.borrow().position == node.borrow().position {
            return Some(vec_node.clone());
        }
    }
    None
}
fn get_end_field(field: &[Vec<u8>]) -> (usize, usize) {
    for (y, row) in field.iter().enumerate() {
        for (x, element) in row.iter().enumerate() {
            if element == &b'X' {
                return (y, x);
            }
        }
    }
    unreachable!("Field must have End Point defined.\nX field must be defined!");
}

//public API
pub fn path_finder(maze: &str) -> Result<Vec<(usize, usize)>, &'static str> {

    let field = maze
        .split('\\')
        .map(|slice| slice.bytes().collect())
        .collect::<Vec<Vec<u8>>>();

    // Create Maze struct, to store dimensions, start & end positions, wall char.
    let (y_size, x_size) = (field.len(), field[0].len());
    let end_node = Node::new((y_size - 1, x_size - 1), get_end_field(&field));
    let mut start_node = Node::new((0, 0), get_end_field(&field));
    // Note: All above should be stored inside Maze struct

    // Note: This all can be removed, we will calculate costs for our start node inside constructor
    start_node.h_cost = ((((end_node.position.1 - start_node.position.1) * 10) * 2
        + ((end_node.position.0 - start_node.position.0) * 10) * 2) as f64)
        .sqrt() as usize;

    // Note: This could be removed with HashMap that will hash nodes based on their position.
    // Note: But we will need to modify parent/previous node field.
    let mut open: NodeVector = vec![Rc::new(RefCell::new(start_node))];
    let mut closed: NodeVector = vec![];


    while !open.is_empty() {
        let current = min_price_node(&open);
        open.retain(|node| node.borrow().position != current.borrow().position);

        for neighbour in current.borrow().neighbours(&field) {
            if neighbour.borrow().position == end_node.position {
                let mut path_vec: Vec<(usize, usize)> = vec![];
                let mut current_node = current.borrow().clone();
                loop {
                    if current_node.parent.is_some() {
                        let new_node = current_node.parent.unwrap().clone();
                        path_vec.push(current_node.position);
                        current_node = new_node.borrow().clone();
                    } else {
                        path_vec.push((0, 0));
                        return Ok(path_vec.into_iter().rev().collect());
                    }
                }
            } else {
                neighbour.borrow_mut().calculate_cost(&current, &end_node)
            }

            if contains_node(&open, &neighbour)
                .is_some_and(|node| node.borrow().f_cost < neighbour.borrow().f_cost)
                || contains_node(&closed, &neighbour)
                    .is_some_and(|node| node.borrow().f_cost < neighbour.borrow().f_cost)
            {
                continue;
            } else {
                open.push(Rc::clone(&neighbour))
            }
        }
        closed.push(Rc::clone(&current));
    }
    Err("Maze is not solvable!")
}
