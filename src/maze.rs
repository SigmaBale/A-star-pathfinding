#![allow(dead_code)]
use crate::error::{Error, ErrorKind::*};
use crate::node::{Node, Position, Priority};
use priority_queue::PriorityQueue;
use std::collections::{HashSet, VecDeque};
use std::default::Default;
use std::fs;

// Colours.
const PATH_COLOUR: &str = "\x1B[92m";
const WALL_COLOUR: &str = "\x1B[91m";
const START_COLOUR: &str = "\x1B[1;93m";
const END_COLOUR: &str = "\x1B[1;93m";
const RESET: &str = "\x1B[0m";

pub type Result<T> = std::result::Result<T, Error>;

/// `Path` is wrapper around the shortest path of the maze.
///
/// Shortest path is represented as a `VecDeque` of a tuple (`usize, usize`) elements that represent coordinates.
struct Path {
    fields: VecDeque<(usize, usize)>,
}

/// [`Maze`] is a core type of this crate with basic API for customizing start, end, separator and wall symbols and some other accessories.
///
/// Once constructed it can give out basic information of our maze parameters, and can parse any `.txt` file.
pub struct Maze {
    maze: Vec<Vec<char>>,
    start: Option<Position>,
    pub(crate) end: Option<Position>,
    path: Option<Path>,
    start_char: char,
    end_char: char,
    wall_char: char,
    path_char: char,
    separator: char,
}

impl Maze {
    /// Constructs a new [`Maze`], by default maze is empty and needs to be set, using `set` method.
    /// `Start` and `End` are [`None`] by default.
    ///
    /// First set your maze using `set`, if your maze contains walls/blockades use `walls_char`
    /// to set symbol that represents them inside your text file (by default this is set to `'W'`).
    /// After that set your `start` and `end` of your maze.
    ///
    /// Setting up start and end of your maze:
    /// - Set your `start` and `end` symbols using `set_start_char` and `set_end_char`.
    ///   Then use those characters inside your text file (by default start is set to `'S'`, end to `'E'`).
    ///
    /// If your file contains multiple start symbols, then the **first appearing** start from text file will be chosen!
    ///
    /// If your file contains multiple end symbols, then the **first appearing** end from text file will be chosen!
    ///
    /// # Examples
    /// ```no_run
    /// fn main() -> Result<(), Error> {
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

    /// Parses the maze into two-dimensional [`Vec`].
    ///
    /// Text file should be properly formatted for parsing to pass.
    /// Fields can be represented with any value in the Unicode codespace; that is, the range of integers from 0 to 10FFFF16.
    ///
    /// To insert new row put '\\' (by default) at the end of the row, or you can set your own separator using `set_separator` method.
    ///
    /// **Make sure that each row is the same length! [Example](https://textdoc.co/EHDkyVKueSNRv7Ao)** (keep note that walls/blockades in the example are denoted with `'W'`).
    ///
    /// # Arguments
    ///
    /// `path` - Filepath of text file that holds the data to construct the maze.
    ///
    /// # Errors
    /// 
    /// Returns [`Error`](crate::error::Error) if it fails to parse the text file.
    /// 
    /// # Examples
    ///
    /// ```no_run
    /// fn main() -> Result<(), Error> {
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
    /// Set splits when it finds newline character.
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

    /// Sets the symbol of walls that will be inside the text file.
    pub fn set_walls_char(mut self, symbol: char) -> Self {
        self.wall_char = symbol;
        self
    }

    /// Sets the symbol of start field that will be inside the text file.
    pub fn set_start_char(mut self, symbol: char) -> Self {
        self.start_char = symbol;
        self.calculate_start();
        self
    }

    /// Sets the symbol of end field that will be inside the text file.
    pub fn set_end_char(mut self, symbol: char) -> Self {
        self.end_char = symbol;
        self.calculate_end();
        self
    }

    /// Sets the symbol for path.
    pub fn set_path_char(mut self, symbol: char) -> Self {
        self.path_char = symbol;
        self
    }

    /// Sets the symbol that marks start of new row inside the text file.
    pub fn set_separator(mut self, symbol: char) -> Self {
        self.separator = symbol;
        self
    }

    /// Returns current path character.
    pub fn path_char(&self) -> char {
        self.path_char
    }

    /// Returns [`char`] that represents the wall inside the text.
    pub fn wall(&self) -> char {
        self.wall_char
    }

    /// Returns reference to formatted maze.
    pub fn field(&self) -> &[Vec<char>] {
        &self.maze
    }

    /// Returns current end [`char`].
    pub fn end_char(&self) -> char {
        self.end_char
    }

    /// Returns current start [`char`].
    pub fn start_char(&self) -> char {
        self.start_char
    }

    /// Returns current separator [`char`].
    pub fn separator_char(&self) -> char {
        self.separator
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

    /// Sets the path in our maze (tries to solve the maze).
    ///
    /// # Errors
    /// If symbols for `start`/`end` are not found inside the text file.
    /// 
    /// If it is impossible to solve the maze.
    ///
    /// If `start`, `end`, `separator` or `wall` share the same character, it will also return [`Error`].
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

    /// Returns [`Vec`] that represents the shortest path from `Start` to the `End`
    ///
    /// # Errors 
    /// If [`Maze`] is not solved.
    pub fn get_path(&self) -> Result<Vec<(usize, usize)>> {
        if let Some(path) = &self.path {
            let vec = path.fields.iter().copied().collect::<Vec<_>>();
            Ok(vec)
        } else {
            Err(MazeNotSolved.into())
        }
    }

    /// Prints the solved [`Maze`], path is marked with `path_char`.
    ///
    /// # Errors
    /// If [`Maze`] is not solved.
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

    /// Prints the parsed [`Maze`].
    ///
    /// # Errors
    /// If [`Maze`] is not set.
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