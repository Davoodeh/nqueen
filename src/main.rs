//! N-Queen problem solution with heuristic ways.
//!
//! This implementation implements a heuristic function based on the number of threats queens pose
//! to each other (includes indirect threats through each other).
//!
//! The implementation runs a VERY greedy search which tries to move the most threatened peice `n`
//! times only if lowers the heuristic. If after a `z` number of tries (defined in
//! `lower_heuristic`) fails to do so, the program panics.
//!
//! This is a homework example of a "heuristic function implementation" not an attempt to solve the
//! N-Queens.

use nqueen::{Board, Point};

/// Move a piece the most checked only if the heuristic shows a lower value.
///
/// Breaks after fixed number of attempts.
///
/// The checks/threats count is the heuristic function in this implementation.
fn lower_heuristic(board: &mut Board) -> Result<(usize, usize, Point, Point), &'static str> {
    const MAX_ATTEMPTS: usize = 1000000;
    for _ in 0..MAX_ATTEMPTS {
        let pre_h = board.checks_count();
        let (from, to) = board.move_most_checked();
        let post_h = board.checks_count();
        if pre_h < post_h {
            board.mov(&to, &from).unwrap();
        } else {
            return Ok((pre_h, post_h, from, to));
        }
    }
    Err("Got stuck in a local minima")
}

fn main() {
    let n = std::env::args()
        .nth(1)
        .expect("Expected an input as the number of Queens (N)")
        .to_owned()
        .parse::<usize>()
        .expect("Invalid number as the input.");

    if n < 4 {
        panic!("Given number must be bigger than 4");
    }

    let mut board = Board::new(n).init_n_queens().unwrap();
    let max_h = board.max_checks();
    println!("Initial heuristic: {}/{}", board.checks_count(), max_h);

    const MAX_MOVES: usize = 100000;
    println!("Not printing random moves without a heustiric change");
    for i in 0..MAX_MOVES {
        let pre_move = board.to_string();
        let (pre_h, h, from, to) = lower_heuristic(&mut board)
            .expect("Expected to lower the heuristic by playing moving the most checked queen");
        let progress = pre_h - h;
        if progress > 0 {
            println!(
                "Move #{}: the most checked queen {} -> {} (benefit: -{}h, total: {}h)",
                i, from, to, progress, h
            );
            let post_move = board.to_string();
            let pre = pre_move.split("\n").collect::<Vec<&str>>();
            let post = post_move.split("\n").collect::<Vec<&str>>();

            for i in 0..(pre.len() - 1)
            /* last line is empty */
            {
                println!("{}  -->  {}", pre[i], post[i]);
            }
            println!("\n{}\n", "#".repeat(79));
        }
        if h == 0 {
            println!("{}", board);
            println!("SOLVED!");
            break;
        }
    }
}
