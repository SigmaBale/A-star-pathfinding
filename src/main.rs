use astar::pathfinding::path_finder;
use std::process;
use std::fs;

fn main() {

    let maze = fs::read_to_string("maze.txt").unwrap();


    let path = path_finder(&maze).unwrap_or_else(|e| {
        println!("Error while trying to solve the maze: {e}");
        process::exit(1);
    });

    print_path(path, &maze);
}

fn print_path(path: Vec<(usize, usize)>, maze: &str) {
    let mut maze: Vec<Vec<u8>> = maze
        .split('\\')
        .map(|slice| slice.bytes().collect())
        .collect::<Vec<Vec<u8>>>();

    for (y, x) in path {
        maze[y][x] = b'+'; //
    }

    println!("\n\nSOLVED MAZE, SHORTEST PATH");
    for row in maze {
        println!(r"{}", String::from_utf8_lossy(&row));
    }
    println!("\n\n");
}