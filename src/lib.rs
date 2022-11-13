pub(crate) mod error;
/// This crate provides basic API for solving 2D mazes inside a text file.
///
/// User can use custom characters for marking start and end positions, row separator
/// if needed and also path character for when maze is solved to be printed.
///
/// Maze is directly loaded from text file and is solved using Euclidian Distance heuristic.
///
/// It uses 8 directions of movement (straight and diagonal).
///
/// For now it only contains **A*** (A-star) pathfinder and it is guaranteed to find the shortest possible path.
///
/// *Note:* *It also uses `extern` crate `priority_queue`.*
pub(crate) mod maze;
pub(crate) mod node;

pub use maze::Maze;
