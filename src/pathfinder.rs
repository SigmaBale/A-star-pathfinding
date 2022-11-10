#![allow(dead_code)]
use crate::error::{Error, ErrorKind};
use priority_queue::PriorityQueue;
use std::collections::{HashSet, VecDeque};
use std::default::Default;
use std::fs;
use std::hash::{Hash, Hasher};

// Set whichever colour you like more.
const PATH_COLOUR: &str = "\x1B[92m";
const WALL_COLOUR: &str = "\x1B[91m";
const START_COLOUR: &str = "\x1B[93m";
const END_COLOUR: &str = "\x1B[36m";
const RESET: &str = "\x1B[0m";

/// Core module that contains public `Maze` struct to setup and load the maze.
pub mod maze {

    use super::*;
    use ErrorKind::*;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
    struct Position((usize, usize));

    /// Wrapper around `f_cost` that represents priority inside the `PriorityQueue`.
    ///
    /// It has custom implementation of `PartialOrd` and `Ord` traits to provide correct functionality when getting
    /// popped out of a priority queue.
    struct Priority(usize);

    /// `Path` is wrapper around the shortest path of the maze.
    ///
    /// Shortest path is represented as a `VecDeque` of a tuple (`usize, usize`) elements that represent coordinates.
    struct Path {
        fields: VecDeque<(usize, usize)>,
    }

    /// Node represents each field in 2D maze, it contains `Position` and costs/weights.
    ///
    /// It also contains heap allocation of its parent/previous `Node` that "discovered" it.
    /// This is due to change, if arena allocator gets implemented.
    #[derive(Clone)]
    struct Node {
        position: Position,
        g_cost: usize,
        h_cost: usize,
        previous: Option<Box<Node>>,
    }

    /// Maze type that is constructed from the `.txt` file using `set` or `set_inline`.
    ///
    /// It has basic API for customization of start, end, separator and wall symbols and some other accessories.
    ///
    /// Once constructed it can give out basic information of our maze parameters.
    pub struct Maze {
        maze: Vec<Vec<char>>,
        start: Option<Position>,
        end: Option<Position>,
        path: Option<Path>,
        start_char: char,
        end_char: char,
        wall_char: char,
        path_char: char,
        separator: char,
    }

    impl Maze {
        /// Constructs a new `Maze`, by default maze is empty and needs to be set, using `set` method.
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
        /// ```no_run, compile_fail
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
                path: None,
                start_char: 'S',
                end_char: 'E',
                wall_char: 'W',
                path_char: 'X',
                separator: '\\',
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
        /// ```no_run, compile_fail
        /// fn main() -> Result<(), &str> {
        ///     // Lets say new_maze.txt contained this text:
        ///     // ".../.../..."
        ///     let mut maze = Maze::new().set("new_maze.txt");
        ///
        ///     assert_eq!(&[vec!['.', '.', '.']; 3], maze.field())
        /// }
        /// ```
        pub fn set_inline(mut self, path: &str) -> Result<Self> {
            if let Ok(maze) = fs::read_to_string(path) {
                let maze = maze
                    .trim()
                    .split(self.separator)
                    .map(|slice| slice.chars().collect())
                    .collect::<Vec<Vec<char>>>();

                self.maze = maze;
                self.calculate_start();
                self.calculate_end();

                Ok(self)
            } else {
                Err(InvalidFilePath.into())
            }
        }

        /// Same as `set_inline`, if you are not using seperator to split into rows, then use set.
        ///
        /// Set splits when it finds whitespace char, so your text file can look like this:
        ///
        /// S...................
        ///
        /// ....................
        ///
        /// ...................E
        ///
        /// **Each new line is automatically formatted to represent new row.**
        pub fn set(mut self, path: &str) -> Result<Self> {
            if let Ok(maze) = fs::read_to_string(path) {
                let maze = maze
                    .split_whitespace()
                    .map(|slice| slice.chars().collect())
                    .collect::<Vec<Vec<char>>>();

                self.maze = maze;
                self.calculate_start();
                self.calculate_end();

                Ok(self)
            } else {
                Err(InvalidFilePath.into())
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

        /// Builder style method you can chain with other builder methods.
        ///
        /// Sets the symbol for path.
        pub fn set_path_char(mut self, symbol: char) -> Self {
            self.path_char = symbol;
            self
        }

        /// Builder style method you can chain with other builder methods.
        ///
        /// Sets the symbol that marks start of new row inside the text file.
        pub fn set_separator(mut self, symbol: char) -> Self {
            self.separator = symbol;
            self
        }

        /// Returns current path character.
        pub fn path_char(&self) -> char {
            self.path_char
        }

        /// Returns `char` that represents the wall inside the text.
        pub fn wall(&self) -> char {
            self.wall_char
        }

        /// Returns reference to formatted maze.
        pub fn field(&self) -> &[Vec<char>] {
            &self.maze
        }

        /// Returns maze length by number of collumns.
        pub fn x_len(&self) -> usize {
            self.maze[0].len()
        }

        /// Returns maze length by number of rows.
        pub fn y_len(&self) -> usize {
            self.maze.len()
        }

        /// Returns maze dimensions `(x_len, y_len)`.
        pub fn dimensions(&self) -> (usize, usize) {
            (self.x_len(), self.y_len())
        }

        /// Tries to solve the maze.
        ///
        /// If symbols for `start`/`end` are not found inside the text file, it will return error.
        ///
        /// If it is impossible to solve the maze, it will return error.
        ///
        /// If `start`, `end`, `separator` or `wall` share the same character, it will return error.
        ///
        /// Otherwise it sets path in our maze.
        pub fn try_solve(&mut self) -> Result<()> {
            if let (Some(start), Some(end)) = (self.start, self.end) {
                if self.are_chars_invalid() {
                    return Err(InvalidCharacters.into());
                }

                let start_node = Node {
                    position: start,
                    g_cost: 0,
                    h_cost: Node::heuristic(start, end),
                    previous: None,
                };
                let priority = Priority(start_node.f_cost());

                let mut open: PriorityQueue<Node, Priority> =
                    PriorityQueue::from(vec![(start_node, priority)]);
                let mut closed: HashSet<Position> = HashSet::new();

                while !open.is_empty() {
                    let current = open.pop().unwrap();

                    if current.0.position.xy() == end.xy() {
                        let mut path = Path {
                            fields: VecDeque::from(vec![current.0.position.xy_usize()]),
                        };
                        let mut curr = current.0.previous;

                        while let Some(node) = curr.take() {
                            path.fields.push_front(node.position.xy_usize());
                            curr = node.previous;
                        }

                        self.path = Some(path);
                        return Ok(());
                    }
                    for mut neighbour in current.0.neighbours(self) {
                        let f_cost = neighbour.f_cost();

                        if closed.get(&neighbour.position).is_some() {
                            continue;
                        } else if let Some(node) = open.get(&neighbour) {
                            if node.0.lower_cost(&neighbour) {
                                continue;
                            } else {
                                neighbour.previous = Some(Box::new(current.0.clone()));
                                open.push(neighbour, Priority(f_cost));
                            }
                        } else {
                            neighbour.previous = Some(Box::new(current.0.clone()));
                            open.push(neighbour, Priority(f_cost));
                        }
                    }
                    closed.insert(current.0.position);
                }
                Err(MazeIsNotSolvable.into())
            } else {
                Err(StartEndNotSet.into())
            }
        }

        /// Returns `Vec` that represents the shortest path from `Start` to the `End`
        ///
        /// If maze is not solved it will return `Err`. You must first `try_solve` the maze.
        pub fn get_path(&self) -> Result<Vec<(usize, usize)>> {
            if let Some(path) = &self.path {
                let vec = path.fields.iter().copied().collect::<Vec<_>>();
                Ok(vec)
            } else {
                Err(MazeNotSolved.into())
            }
        }

        /// Prints the solved maze, path is marked with `path_char`.
        ///
        /// If maze is not solved, it will return `Err`. You must first `try_solve` the maze.
        pub fn print_path(&self) -> Result<()> {
            if self.path.is_some() {
                let x_str_len = self.x_len().to_string().len() as i32;
                let x_len = (self.x_len() as i32 - x_str_len).unsigned_abs() as usize;

                let y_str_len = self.y_len().to_string().len() as i32;
                let y_len = (self.y_len() as i32 - y_str_len).unsigned_abs() as usize;

                let horizontal = format!("<{:-^x_len$}>", self.x_len());
                let vertical: Vec<char> = format!("^{:|^y_len$}v", self.y_len()).chars().collect();
                let slice = &vertical[..];

                println!("{}", horizontal);
                for (y, row) in self.maze.iter().enumerate() {
                    for (x, char) in row.iter().copied().enumerate() {
                        if char == self.wall_char {
                            print!("{}{char}{}", WALL_COLOUR, RESET)
                        } else if char == self.start_char {
                            print!("{}{char}{}", START_COLOUR, RESET)
                        } else if char == self.end_char {
                            print!("{}{char}{}", END_COLOUR, RESET)
                        } else if self.path.as_ref().unwrap().fields.contains(&(x, y)) {
                            print!("{}{}{}", PATH_COLOUR, self.path_char, RESET)
                        } else {
                            print!("{char}")
                        }
                    }
                    println!(" {}", slice[y]);
                }

                Ok(())
            } else {
                Err(MazeIsNotSet.into())
            }
        }

        /// Prints the maze that is loaded from the text file.
        ///
        /// If maze is not loaded then it will return `Err`. You must first `set` the maze.
        pub fn print_maze(&self) -> Result<()> {
            if !self.maze.is_empty() {
                let x_str_len = self.x_len().to_string().len() as i32;
                let x_len = (self.x_len() as i32 - x_str_len).unsigned_abs() as usize;

                let y_str_len = self.y_len().to_string().len() as i32;
                let y_len = (self.y_len() as i32 - y_str_len).unsigned_abs() as usize;

                let horizontal = format!("<{:-^x_len$}>", self.x_len());
                let vertical: Vec<char> = format!("^{:|^y_len$}v", self.y_len()).chars().collect();
                let slice = &vertical[..];

                println!("{}", horizontal);
                for (y, row) in self.maze.iter().enumerate() {
                    for char in row.iter().copied() {
                        if char == self.wall_char {
                            print!("{}{char}{}", WALL_COLOUR, RESET)
                        } else if char == self.start_char {
                            print!("{}{char}{}", START_COLOUR, RESET)
                        } else if char == self.end_char {
                            print!("{}{char}{}", END_COLOUR, RESET)
                        } else {
                            print!("{char}")
                        }
                    }
                    println!(" {}", slice[y]);
                }
                println!("\n\n");
                Ok(())
            } else {
                Err(MazeIsNotSet.into())
            }
        }

        /// Helper function for checking if all characters are unique.
        fn are_chars_invalid(&self) -> bool {
            self.end_char == self.start_char
                || self.start_char == self.separator
                || self.end_char == self.separator
                || self.wall_char == self.separator
                || self.wall_char == self.start_char
                || self.wall_char == self.end_char
        }

        /// Helper function for finding start character and setting start position.
        fn calculate_start(&mut self) {
            for (i, row) in self.maze.iter().enumerate() {
                let start = row
                    .iter()
                    .enumerate()
                    .find(|(_, char)| **char == self.start_char);
                if let Some((x_cord, _)) = start {
                    self.start = Some(Position((x_cord, i)));
                    return;
                }
            }
        }

        /// Helper function for finding end character and setting end position.
        fn calculate_end(&mut self) {
            for (i, row) in self.maze.iter().enumerate() {
                let start = row
                    .iter()
                    .enumerate()
                    .find(|(_, char)| **char == self.end_char);
                if let Some((x_cord, _)) = start {
                    self.end = Some(Position((x_cord, i)));
                    return;
                }
            }
        }
    }

    impl Default for Maze {
        fn default() -> Self {
            Self::new()
        }
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

        fn neighbours(&self, maze: &Maze) -> Vec<Node> {
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

        fn lower_cost(&self, neighbour: &Node) -> bool {
            self.f_cost() < neighbour.f_cost()
                || (self.h_cost < neighbour.h_cost && self.f_cost() == neighbour.f_cost())
        }

        fn heuristic(position: Position, end: Position) -> usize {
            let a = (end.x() - position.x()).abs() * 10;
            let b = (end.y() - position.y()).abs() * 10;
            let c = a.pow(2) + b.pow(2);
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
            } else {
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
            self.position.0 == other.position.0
        }
    }

    impl Eq for Node {}

    impl Hash for Node {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.position.0.hash(state);
        }
    }

    impl Position {
        fn x(&self) -> isize {
            self.0 .0 as isize
        }

        fn y(&self) -> isize {
            self.0 .1 as isize
        }

        fn xy(&self) -> (isize, isize) {
            (self.0 .0 as isize, self.0 .1 as isize)
        }

        fn x_usize(&self) -> usize {
            self.0 .0
        }

        fn y_usize(&self) -> usize {
            self.0 .1
        }

        fn xy_usize(&self) -> (usize, usize) {
            (self.0 .0, self.0 .1)
        }
    }

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
}
