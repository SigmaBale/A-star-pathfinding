use astar::pathfinder::maze::Maze;
use std::process;

// Example:
fn main() {
    let mut maze = Maze::new().set("maze.txt").unwrap_or_else(|e| {
        println!("{e}");
        process::exit(1);
    });

    maze.print_maze().unwrap_or_else(|e| {
        println!("{e}");
        process::exit(1);
    });

    maze.try_solve().unwrap_or_else(|e| {
        println!("{e}");
        process::exit(1);
    });

    maze.print_path().unwrap_or_else(|e| {
        println!("{e}");
        process::exit(1);
    });

    let path = maze.get_path().unwrap();

    println!("\nPath: {:?}", path);
}
