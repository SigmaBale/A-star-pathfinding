#![allow(dead_code)]
use std::fs;

/// Representation of shortest path (solution to the maze). 
pub struct Path {
    path: Vec<(usize, usize)>
}

impl Path {
    fn new(solution: Vec<(usize, usize)>) -> Self {
        Path { path: solution }
    }
}

/// Maze type that is constructed from the maze `.txt` file.
/// 
/// It has basic API for customization of start, end and wall symbols.
/// 
/// Once constructed it can give out basic information of our maze dimensions.
pub struct Maze {
    maze: Vec<Vec<char>>,
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
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
        self
    }

    /// Builder style method that you can chain with other builder methods.
    /// 
    /// Sets the symbol of end field that will be inside the text file.
    pub fn set_end_char(mut self, symbol: char) -> Self {
        self.end_char = symbol;
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

    // pub fn solve(&self) -> Result<>

    fn calculate_start(&mut self) {
        for (i, row) in self.maze.iter().enumerate() {
            let start = row.iter().enumerate().find(|(_, char)| **char == self.start_char);
            if let Some((x_cord, _)) = start {
                self.start = Some((x_cord, i));
            }
        }
    }

    fn calculate_end(&mut self) {
        for (i, row) in self.maze.iter().enumerate() {
            let start = row.iter().enumerate().find(|(_, char)| **char == self.end_char);
            if let Some((x_cord, _)) = start {
                self.end = Some((x_cord, i));
            }
        }
    }
}
