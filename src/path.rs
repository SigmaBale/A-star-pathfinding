#![allow(dead_code)]
use priority_queue::PriorityQueue;
use std::collections::VecDeque;
use std::fs;
use std::hash::{Hash, Hasher};
use std::default::Default;

pub mod maze {

    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct Position((usize, usize));

    struct Priority(usize);

    struct Path {
        fields: VecDeque<(usize, usize)>,
    }

    #[derive(Clone)]
    struct Node {
        position: Position,
        // note: remove g_cost, h_cost to save on memory (cuz padding)
        g_cost: usize, // convert into a method
        h_cost: usize, // convert into a method
        previous: Option<Box<Node>>,
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
        path: Option<Path>,
        start_char: char,
        end_char: char,
        wall_char: char,
        path_char: char,
        separator: char,
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
        /// ```no_run
        /// fn main() -> Result<(), &str> {
        ///     // Lets say new_maze.txt contained this text:
        ///     // ".../.../..."
        ///     let mut maze = Maze::new().set("new_maze.txt");
        ///
        ///     assert_eq!(&[vec!['.', '.', '.']; 3], maze.field())
        /// }
        /// ```
        pub fn set(mut self, path: &str) -> Result<Self, &str> {
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
        /// If symbols for `start`/`end` are not found inside the text file, it will return error.
        ///
        /// If it is impossible to solve the maze, it will return error.
        ///
        /// If `start`, `end`, `separator` or `wall` share the same character, it will return error.
        ///
        /// Otherwise it sets path in our maze.
        pub fn try_solve(&mut self) -> Result<(), &'static str> {
            if let (Some(start), Some(end)) = (self.start, self.end) {
                if self.are_chars_invalid() {
                    return Err("Start, End, Separator and Wall characters have to be different values, they can't share the same character!");
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
                let mut closed: PriorityQueue<Node, Priority> = PriorityQueue::new();

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

                        if let Some(node) = closed.get(&neighbour) {
                            if node.0.lower_cost(&neighbour) {
                                continue;
                            } else {
                                neighbour.previous = Some(Box::new(current.0.clone()));
                                open.push(neighbour, Priority(f_cost));
                            }
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
                    closed.push(current.0, current.1);
                }
                Err("Maze is not solvable, impossible to reach the end! :(")
            } else {
                Err("Start/End is not set, check that your characters match the ones in the text file!")
            }
        }

        pub fn get_path(&self) -> Result<Vec<(usize, usize)>, &'static str> {
            if let Some(path) = &self.path {
                let vec = path.fields.iter().copied().collect::<Vec<_>>();
                Ok(vec)
            } else {
                Err("Path is not found! First solve the maze using `try_solve`.")
            }
        }

        pub fn print_path(&self) -> Result<(), &'static str> {
            if let Some(path) = &self.path {
                let mut maze = self.maze.clone();

                for (x, y) in path.fields.iter().skip(1) {
                    debug_assert!(maze[*y][*x] != self.wall_char);
                    if (*x, *y) == self.end.unwrap().xy_usize() {
                        break;
                    }
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
            } else {
                Err("Path is not found! First solve the maze using `try_solve`.")
            }
        }

        pub fn print_maze(&self) -> Result<(), &'static str> {
            if !self.maze.is_empty() {
                println!("\n\n");
                let maze = self
                    .maze
                    .iter()
                    .map(|row| row.iter().collect::<String>())
                    .collect::<Vec<_>>();

                for row in maze.iter() {
                    println!("{row}");
                }
                println!("\n\n");
                Ok(())
            } else {
                Err("Maze is not set! Set your maze using `set`!")
            }
        }

        pub fn wall(&self) -> char {
            self.wall_char
        }

        fn are_chars_invalid(&self) -> bool {
            self.end_char == self.start_char
                || self.start_char == self.separator
                || self.end_char == self.separator
                || self.wall_char == self.separator
                || self.wall_char == self.start_char
                || self.wall_char == self.end_char
        }

        fn end(&self) -> Option<Position> {
            self.end
        }

        fn start(&self) -> Option<Position> {
            self.start
        }

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
                position: position,
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
