#![allow(dead_code)]
use std::fs;
use std::cell::Cell;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::ptr::{self, read};

pub mod maze {
    use super::{BTreeSet, Cell, fs, VecDeque, read, ptr};

    #[derive(Clone, Copy)]
    struct Position((usize, usize));

    struct Path {
        fields: VecDeque<(usize, usize)>
    }

    struct Node {
        position: Position, 
        // note: remove g_cost, h_cost to save on memory (cuz padding)
        g_cost: usize, // convert into a method
        h_cost: usize, // convert into a method
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
        path: Cell<Option<Path>>,
        start_char: char,
        end_char: char,
        wall_char: char,
        path_char: char,
        separator: char
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
                path: Cell::new(None),
                start_char: 'S',
                end_char: 'E',
                wall_char: 'W',
                path_char: 'X',
                separator: '\\'
            }
        }

        /// Formats the maze, returns `Result` in case file path is invalid.
        /// `Err` returns file path that was passed in.
        /// 
        /// Text file should be properly formatted to produce the right format for the maze.
        /// Fields can be represented with any value in the Unicode codespace; that is, the range of integers from 0 to 10FFFF16.
        /// 
        /// To insert new row put '\\' (by default) at the end of the row, or you can set your own separator using `set_separator`.
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
                    .split(self.separator)
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

        pub fn set_separator(mut self, symbol: char) -> Self {
            self.separator = symbol;
            self
        }

        pub fn path_char(&self) -> char {
            self.path_char
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
        /// If some of the parameters for `start`/`end` are not found inside the text file, it will return error.
        /// 
        /// If it is impossible to solve the maze, it will return error.
        /// 
        /// If `start`, `end` or `wall` share the same character, it will return error.
        /// 
        /// Otherwise it returns `Path`.
        pub fn try_solve(&self) -> Result<(), &'static str> {
            if let (Some(start), Some(end)) = (self.start, self.end) {
                if self.are_chars_invalid() {
                    return Err("Start, End and Wall characters have to be different values, they can't be the same character!")
                }

                let mut path = Path { fields: VecDeque::new() };
                let start_node = Node {
                    position: start,
                    g_cost: 0,
                    h_cost: Node::heuristic(start, end),
                    previous: Cell::new(None)
                };

                let mut open: BTreeSet<Node> = BTreeSet::from([start_node]);
                let mut closed: BTreeSet<Node> = BTreeSet::new();
                
                let mut counter = 1;

                while !open.is_empty() {
                    counter += 1;
                    if counter > 60 { break }
                    let current = open.pop_first().unwrap();
                    println!("");
                    println!("Current: {:?}", current.position.xy());
                    println!("Popped value with f_cost = {}", current.f_cost());
                    print!("Open:");
                    for key in open.iter() {
                        print!(" {} -", key.f_cost());
                    }
                    println!("");

                    if current.position.xy() == self.end.unwrap().xy() {
                        let mut curr = current;
                        while let Some(node) = curr.previous.get().take() {
                            debug_assert_ne!(node, ptr::null());
                            unsafe {
                                let prev = read(node);
                                path.fields.push_back(prev.position.0);
                                curr = prev;
                            }
                        }
                        self.path.set(Some(path));
                        return Ok(())
                    } else {
                        for neighbour in current.neighbours(self) {
                            let open_node = open.get(&neighbour);
                            let closed_node = closed.get(&neighbour);

                            if closed_node.is_some() {
                                continue;
                            } else if let Some(node) = open_node {
                                if node.f_cost() < neighbour.f_cost() {
                                    continue;
                                } else {
                                    open.insert(neighbour);
                                }
                            } else {
                                println!("\
                                    Inserting node pos {:?}\nf_cost: {}\nh_cost: {}\ng_cost: {}",
                                    neighbour.position.xy(),
                                    neighbour.f_cost(), 
                                    neighbour.h_cost,
                                    neighbour.g_cost
                                );
                                open.insert(neighbour);
                            }
                        }
                    }
                    closed.insert(current);
                }
                Err("Maze is not solvable, impossible to reach the end! :(")
            }else {
                Err("Start/End is not set, check that your characters match the ones in the text file!")
            }
        }

        pub fn print_path(&self) -> Result<(), &'static str> {
            if let Some(path) = self.path.take() {
                let mut maze = self.maze.clone();

                for (x, y) in path.fields.iter() {
                    maze[*y][*x] = self.path_char;
                }

                let maze = maze
                    .into_iter()
                    .map(|row| row.into_iter().collect::<String>())
                    .collect::<Vec<String>>();

                for row in maze {
                    println!("{row}");
                }

                Ok(())
            }else {
                Err("Path is not found! First solve the maze using `try_solve`.")
            }
        }

        pub fn print_maze(&self) -> Result<(), &'static str> {
            if !self.maze.is_empty() {
                let maze = 
                    self.maze
                        .clone()
                        .into_iter()
                        .map(|row| row.into_iter().collect::<String>())
                        .collect::<Vec<_>>();

                for row in maze.iter() {
                    println!("{row}");
                }

                Ok(())
            } else {
                Err("Maze is not set! Set your maze using `set`!")
            }
        }
        
        pub fn wall(&self) -> char {
            self.wall_char
        }

        pub(crate) fn are_chars_invalid(&self) -> bool {
            self.end_char == self.start_char
                || self.start_char == self.separator
                || self.end_char == self.separator
                || self.wall_char == self.separator
                || self.wall_char == self.start_char
                || self.wall_char == self.end_char
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
                    println!("Found start! Row: {}, Column: {}", i + 1, x_cord + 1);
                    self.start = Some(Position((x_cord, i)));
                    println!("{:?}", self.start.unwrap().xy());
                    return
                }else {
                    println!("Could not find start symbol: {}, in row {}...", self.start_char, i + 1);
                }
            }
        }

        fn calculate_end(&mut self) {
            for (i, row) in self.maze.iter().enumerate() {
                let start = row.iter().enumerate().find(|(_, char)| **char == self.end_char);
                if let Some((x_cord, _)) = start {
                    println!("Found end! Row: {}, Column: {}", i+1, x_cord + 1);
                    self.end = Some(Position((x_cord, i)));
                    println!("{:?}", self.end.unwrap().xy());
                    return
                }else {
                    println!("Could not find end symbol: {}, in row {}...", self.end_char, i + 1);
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
                let nx = self.position.x() + offset_x[i];
                let ny = self.position.y() + offset_y[i];

                if Node::is_valid((nx, ny), maze) {
                    let (nx, ny) = (nx as usize, ny as usize);

                    // SAFETY (unwrap): If we are executing neighbours method it means we are also executing
                    // try_solve for Maze, we already ensured before in try_solve that we return early if end or start is None.
                    let node = Node::new(Position((nx, ny)), self, maze.end().unwrap());
                    // We are never deallocating nodes at least not until program ends, that is how astar alg works, eats your memory. (note: implement allocator arena) 
                    // So we will never point to a freed memory, that way we can later extract our Path, so we are ok with storing a raw pointer.
                    // Because when we later dereference it to get position of that node, pointer will always be valid (it won't point to a freed memory).
                    node.previous.set(Some(self as *const _));
                    neighbours.push(node);

                }else {
                    continue;
                }
            }
            neighbours
        }

        fn heuristic(position: Position, end: Position) -> usize {
            let a = ((end.x() - position.x()) * 10).pow(2);
            let b = ((end.y() - position.y()) * 10).pow(2);
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

            if diagonal_positions.contains(&position.xy()) {
                prev.g_cost + 14
            }else {
                prev.g_cost + 10
            }
        }

        fn f_cost(&self) -> usize {
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
            self.position.xy() == other.position.xy()
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

    impl Position {
        fn x(&self) -> isize {
            self.0.0 as isize
        }

        fn y(&self) -> isize {
            self.0.1 as isize
        }

        fn xy(&self) -> (isize, isize) {
            (self.0.0 as isize, self.0.1 as isize)
        }
    }

}