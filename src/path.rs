#![allow(dead_code)]
use std::fs;
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::collections::BTreeSet;


pub mod maze {
    use super::{Hash, Hasher, BTreeSet, Cell, fs};

    #[derive(Clone, Copy)]
    struct Position((usize, usize));

    pub struct Path {
        path: Vec<(usize, usize)>
    }

    struct Node {
        position: Position,
        g_cost: usize,
        h_cost: usize,
        previous: Cell<Option<*const Node>>
    }

    /// Maze type that is constructed from the maze `.txt` file.
    /// 
    /// It has basic API for customization of start, end and wall symbols.
    /// 
    /// Once constructed it can give out basic information of our maze dimensions.
    pub struct Maze {
        maze: Vec<Vec<char>>,
        start: Option<Position>,
        end: Option<Position>,
        start_char: char,
        end_char: char,
        wall_char: char,
        path_char: char
    }

    impl Maze {
        /// Constructs a new `Maze`, by default maze is empty and needs
        /// to be set, using `set` method. 
        /// `Start` and `End` are `None` by default.
        /// 
        /// First set your maze using `set`, then if your maze contains walls/blockades use `walls_char`
        /// to set symbol that represents them inside your text file (by default this is set to `'W'`).
        /// After that set your `start` and `end` of your maze.
        /// 
        /// Setting up start and end of your maze:
        /// - Set your `start` and `end` symbols using `set_start_char` and `set_end_char`.
        ///   And then just use those characters inside your text file (by default start is set to `'S'`, end to `'E'`).
        /// 
        /// **If your file contains multiple start symbols, then the first appearing start from text file will be chosen!**
        /// 
        /// **If your file contains multiple end symbols, then the first appearing end from text file will be chosen!**
        /// 
        /// # Examples
        /// ```no_run
        /// fn main() -> Result<(), &'static str> {
        ///     let mut maze = Maze::new();
        /// 
        ///     maze.set("maze.txt")?;
        /// 
        ///     let maze = maze
        ///         .set_walls_char('O')
        ///         .set_start_char('G')
        ///         .set_end_char('F');
        /// 
        ///     Ok(())
        /// }
        /// ```
        pub fn new() -> Self {
            Maze {
                maze: vec![vec![]],
                start: None,
                end: None,
                start_char: 'S',
                end_char: 'E',
                wall_char: 'W',
                path_char: 'X'
            }
        }

        /// Formats the maze, returns `Result` in case file path is invalid.
        /// `Err` returns file path that was passed in.
        /// 
        /// Text file should be properly formatted to produce the right format for the maze.
        /// Fields can be represented with any valid utf-8 char. 
        /// 
        /// To insert new row put '\\' (escape char) at the end of the row.
        /// 
        /// **Make sure that each row is the same length! [Example](https://textdoc.co/EHDkyVKueSNRv7Ao)** (keep note that walls/blockades in the example are denoted with `'W'`).
        ///
        /// # Arguments
        ///
        /// `path` - Filepath of text file that holds the data to construct the maze.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// fn main() -> Result<(), &str> {
        ///     // Lets say new_maze.txt contained this text:
        ///     // ".../.../..."
        ///     let mut maze = Maze::new().set("new_maze.txt");
        ///
        ///     assert_eq!(&[vec!['.', '.', '.']; 3], maze.field())
        /// }
        /// ```
        pub fn set<'b>(mut self, path: &'b str) -> Result<Self, &'b str> {
            if let Ok(maze) = fs::read_to_string(path) {
                let maze = maze
                    .split('\\')
                    .map(|slice| slice.chars().collect())
                    .collect::<Vec<Vec<char>>>();

                self.maze = maze;
                self.calculate_start();
                self.calculate_end();

                Ok(self)
            } else {
                Err(path)
            }
        }

        /// Builder style method that you can chain with other builder methods.
        /// 
        /// Sets the symbol of walls that will be inside the text file.
        pub fn set_walls_char(mut self, symbol: char) -> Self {
            self.wall_char = symbol;
            self
        }

        /// Builder style method that you can chain with other builder methods.
        /// 
        /// Sets the symbol of start field that will be inside the text file.
        pub fn set_start_char(mut self, symbol: char) -> Self {
            self.start_char = symbol;
            self.calculate_start();
            self
        }

        /// Builder style method that you can chain with other builder methods.
        /// 
        /// Sets the symbol of end field that will be inside the text file.
        pub fn set_end_char(mut self, symbol: char) -> Self {
            self.end_char = symbol;
            self.calculate_end();
            self
        }

        pub fn set_path_char(mut self, symbol: char) -> Self {
            self.path_char = symbol;
            self
        }

        /// Returns reference to formatted maze.
        pub fn field(&self) -> &[Vec<char>] {
            &self.maze
        }

        /// Returns maze length by number of collumns.
        /// It is called `x_len` because of Cartesian plane.
        pub fn x_len(&self) -> usize {
            self.maze[0].len()
        }

        /// Returns maze length by number of rows.
        /// It is called `y_len` because of Cartesian plane.
        pub fn y_len(&self) -> usize {
            self.maze.len()
        }

        /// Returns maze dimensions `(x_len, y_len)`.
        pub fn dimensions(&self) -> (usize, usize) {
            (self.x_len(), self.y_len())
        }

        /// Tries to solve the maze. 
        /// 
        /// If some of the symbol parameters are not found inside the text file, this will return detailed error.
        pub fn try_solve(&self) -> Result<Path, &'static str> {
            if let (Some(start), Some(end)) = (self.start, self.end) {

                if self.start_char == self.end_char
                    || self.start_char == self.wall_char
                    || self.end_char == self.wall_char
                {
                    return Err("Start, End and Wall characters have to be different values, they can't be the same character!")
                }

                let path = Path { path: Vec::new() };
                let start_node = Node {
                    position: start,
                    g_cost: 0,
                    h_cost: Node::heuristic(start, end),
                    previous: Cell::new(None)
                };
                // Used std library BinaryHeap (max) cuz lazy.
                // Will work for poping smallest f_cost (custom implementation of Ord on Node)
                let mut open: BTreeSet<Node> = BTreeSet::from([start_node]);
                let mut closed: BTreeSet<Node> = BTreeSet::new(); // binary heap mayybe?
                
                while !open.is_empty() {
                    // Safe to unwrap, if we are in loop then open is not empty...
                    // let min_node = open.iter().min_by(|a, b| a.cmp(&b)).unwrap();
                    // let current = open.take(min_node).unwrap();

                    // for neighbour in current.neighbours(self) {
                    //     if open.contains(&neighbour) || closed.contains(&neighbour) {
                    //         continue;
                    //     }else {
                    //         open.insert(neighbour);
                    //     }
                    // }
                }


                Ok(path)
            }else {
                Err("Start/End is not set, check that your character matches the one in the text file!")
            }
        }

        pub fn wall(&self) -> char {
            self.wall_char
        }

        fn end(&self) -> Option<Position> {
            self.end.map(|position| position)
        }

        fn start(&self) -> Option<Position> {
            self.start.map(|position| position)
        }

        fn calculate_start(&mut self) {
            for (i, row) in self.maze.iter().enumerate() {
                let start = row.iter().enumerate().find(|(_, char)| **char == self.start_char);
                if let Some((x_cord, _)) = start {
                    self.start = Some(Position((x_cord, i)));
                }
            }
        }

        fn calculate_end(&mut self) {
            for (i, row) in self.maze.iter().enumerate() {
                let start = row.iter().enumerate().find(|(_, char)| **char == self.end_char);
                if let Some((x_cord, _)) = start {
                    self.end = Some(Position((x_cord, i)));
                }
            }
        }
    }

    impl Node {
        fn new(position: Position, previous: &Node, end: Position) -> Self {
            let mut node = Node {
                position: position,
                g_cost: 0,
                h_cost: 0,
                previous: Cell::new(None),
            };

            node.h_cost = Node::heuristic(node.position, end);
            node.g_cost = Node::g_cost(position, previous);
            node
        }

        fn neighbours(&self, maze: &Maze) -> Vec<Node> {
            let offset_x: [isize; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
            let offset_y: [isize; 8] = [0, -1, -1, -1, 0, 1, 1, 1];
            
            let mut neighbours = Vec::new();

            for i in 0..8 {
                // Hopefully this is not giga maze of hell :))) :/
                // Max number of nodes: 2^63 − 1 (64-bit targets) meow, bark bark.
                let nx = self.position.x() as isize + offset_x[i];
                let ny = self.position.y() as isize + offset_y[i];

                if Node::is_valid((nx, ny), maze) {
                    let (nx, ny) = (nx as usize, ny as usize);

                    // SAFETY (unwrap): If we are executing neighbours method it means we are also executing
                    // try_solve for Maze, we already ensured before in try_solve that we return early if end or start is None.
                    let node = Node::new(Position((nx, ny)), self, maze.end().unwrap());
                    // We are never deallocating nodes at least not until program ends, that is how astar alg works, eats your memory. (note: implement arena) 
                    // So we will never point to a freed memory, that way we can later extract our Path, so we are ok with storing a raw pointer.
                    // Because when we later dereference it to get position of that node, pointer will always be valid (it won't point to a freed memory).
                    node.previous.set(Some(self as *const _)); // if value gets moved pointer follows?? WARNING!
                    neighbours.push(node);
                }else {
                    continue;
                }
            }
            neighbours
        }

        fn heuristic(position: Position, end: Position) -> usize {
            let a = ((position.x() - end.x()) * 10).pow(2);
            let b = ((position.y() - end.y()) * 10).pow(2);
            let c = a + b;
            (c as f64).sqrt() as usize
        }

        fn g_cost(position: Position, prev: &Node) -> usize {
            let diagonal_positions = vec![
                (prev.position.x() - 1, prev.position.y() - 1),
                (prev.position.x() - 1, prev.position.y() + 1),
                (prev.position.x() + 1, prev.position.y() - 1),
                (prev.position.x() + 1, prev.position.y() + 1),
            ];

            if diagonal_positions.contains(&position.0) {
                prev.g_cost + 14
            }else {
                prev.g_cost + 10
            }
        }

        fn f_cost(&self) -> usize {
            self.g_cost + self.h_cost
        }

        fn is_valid(position: (isize, isize), maze: &Maze) -> bool {
            if position.0 >= maze.x_len() as isize
                || position.0 < 0
                || position.1 >= maze.y_len() as isize
                || position.1 < 0
                || maze.field()[position.1 as usize][position.0 as usize] == maze.wall()
            {
                false
            }
            else { true }
        }
    }

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            self.position.xy() == other.position.xy()
            && self.f_cost() == other.f_cost()
        }
    }
    
    impl Eq for Node {}

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.f_cost().partial_cmp(&other.f_cost())
        }
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.f_cost().cmp(&other.f_cost())
        }
    }

    impl Hash for Node {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.position.xy().hash(state);
            self.f_cost().hash(state);
        }
    }

    impl Position {
        fn x(&self) -> usize {
            self.0.0
        }

        fn y(&self) -> usize {
            self.0.1
        }

        fn xy(&self) -> (usize, usize) {
            (self.0.0, self.0.1)
        }
    }

}