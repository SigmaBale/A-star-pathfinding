use astar::path::maze::Maze;
use std::process;

fn main() {
    let maze = Maze::new().set("maze.txt")
        .unwrap_or_else(|e| {
            println!("{e}");
            process::exit(1);
        });

    maze.print_maze()
        .unwrap_or_else(|e| {
            println!("{e}");
            process::exit(1);
        });

    maze.try_solve()
        .unwrap_or_else(|e| {
            println!("{e}");
            process::exit(1);
        });

    maze.print_path()
        .unwrap_or_else(|e| {
            println!("{e}");
            process::exit(1);
        });
}