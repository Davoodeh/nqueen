//! Solves N-Queens problem with different basic algorithms.
//!
//! See `Mode` enum for the various algorithms implmemented.

use rand::prelude::*;

use nqueen::{Board, Point};

/// The possible modes/algorithms of the program.
///
/// NOTE I'm well aware that one should not write everything in a single function. But I saw fit to
/// implement ONE and ONLY ONE `*-solution` method for each variant. This is a deliberate choice.
enum Mode {
    /// See `random_solution` for more details.
    Random,
    /// See `genetic_solution` for more details.
    Genetic,
}

impl Mode {
    /// Solve the selected mode.
    pub fn solve(&self, n: usize) {
        match self {
            Self::Random => Self::random_solution(n),
            Self::Genetic => Self::genetic_solution(n),
        }
    }

    /// Solve the problem using the genetics algorithm.
    pub fn genetic_solution(n: usize) {
        const POPULATION: usize = 30;
        /// How many will make it to the next generation. NOTE LOWER THAN POPULATION
        const SURVIVORS: usize = 6;
        /// How many get combined to for the next child.
        const PARENTS: usize = 2;
        /// The mutation chance (out of 100).
        const MUTATION_CHANCE: usize = 10;
        /// Number of moves in each generation.
        const MOVES_IN_GENERATION: usize = 3;
        /// Number of maximum generations.
        const GENERATIONS: usize = 1000;
        // Read more: aggressive genes, stable marraige/shuffling parents, twin/triplet/etc chance.

        assert!(
            PARENTS < n,
            "The chess board size is smaller than the number of parents.\
             To keep the code simple, this is not supported."
        );

        let mut rng = thread_rng();

        // Holds the boards.
        let mut env = vec![Board::new(n); POPULATION]
            .into_iter()
            .map(|i| i.init_n_queens().unwrap())
            .collect::<Vec<Board>>();
        /// Create the primitive/initial boards, the natives of the env.
        // Print all the heuristics of the boards in the env
        let all_heuristics =
            |env: &Vec<Board>| env.iter().map(|i| i.checks_count()).collect::<Vec<usize>>();

        println!(
            "Environment details:\n\
             - initial population: {POPULATION}\n\
             - each generation's max survivors: {SURVIVORS}\n\
             - number of parents required for a child: {PARENTS}\n\
             - mutation chance: {MUTATION_CHANCE}%\n\
             - moves before a generation dies out: {MOVES_IN_GENERATION}\n\
             - maximum generations: {GENERATIONS}\n\
            "
        );

        'time: for generation in 0..GENERATIONS {
            println!("Generation #{}", generation);
            println!("This generation's heuristics: {:?}", all_heuristics(&env));

            // Let them live their lives
            for board in env.iter_mut() {
                for _ in 0..MOVES_IN_GENERATION {
                    Self::lower_heuristic(board);
                }
            }

            // Sort by fitness
            env.sort_by(|a, b| a.checks_count().cmp(&b.checks_count()));
            println!(
                "This generation's heuristics after {} moves: {:?}",
                MOVES_IN_GENERATION,
                all_heuristics(&env)
            );

            // Pick this generation of survivors and check for the fittest or continue.
            if env.len() < SURVIVORS {
                panic!("Everybody died!");
            }
            let survivors = &env[0..SURVIVORS].to_vec();
            let survivors_heuristics = all_heuristics(&survivors);
            println!(
                "The {} survivors of this generation are: {:?}",
                SURVIVORS, survivors_heuristics,
            );
            if survivors_heuristics[0] == 0 {
                println!("The fittest was found!");
                println!("{}", survivors[0]);
                break 'time;
            }

            env = vec![];
            // Make children from the survivors
            'child_production: for i in 0..(POPULATION / PARENTS) {
                // println!("Managing the couple #{}", i);
                // Choose some parents to make a child from them
                let mut parents = Vec::<Board>::with_capacity(PARENTS);
                for i in 0..PARENTS {
                    // NOTE Since this is not important, we leave the chance for a board to have
                    // children from itself.
                    let randomly_picked_parent = survivors[rng.gen_range(0..SURVIVORS)].clone();
                    parents.push(randomly_picked_parent);
                }

                // Create the child from their stats and distribute the information/genes equally.
                // In this example implementation, the gene split rate is uniform meaning all
                // parents pass equal amount of genes to their children.
                let mut child_genes = Vec::<Point>::with_capacity(n);
                let gene_portions = n / PARENTS; // number of genes from each parent
                let last_parent_extra_passing = n % PARENTS; // leftover genes for the last parent
                for i in 0..PARENTS {
                    // println!("Subject parent #{}: {}", i, parents[i].queens_display());
                    let mut extra_genes = 0;
                    // As for the last parent, give all the remaining genes to the child (may be
                    // more than others).
                    if i == PARENTS - 1 {
                        extra_genes = last_parent_extra_passing;
                    }
                    for j in 0..(gene_portions + extra_genes) {
                        // Pick the first n/PARENTS genes from the first parent then the next till
                        // one is left
                        let gene = parents[i].queens()[(i * gene_portions) + j].clone();
                        // println!("Inheriting {} from parent #{}", gene, i);
                        child_genes.push(gene);
                    }
                }

                // Mutate the child genes (move the pieces randomly).
                // If two pieces collide, mark the board as invalid/cancerous/high-cost (removes it
                // from the next generation).
                for i in child_genes.iter_mut() {
                    let pre = i.clone();
                    let chance = rng.gen_range(0..100);
                    let mutated_coord = rng.gen_range(0..n);
                    // println!(
                    //     "Mutation: {}>={} ; Mutated Coord: {}",
                    //     MUTATION_CHANCE,
                    //     chance,
                    //     (mutated_coord + 1) // Point is +1 in its diplay
                    // );
                    if chance < (MUTATION_CHANCE / 2) {
                        // 50% to mutate the row
                        i.row = mutated_coord;
                    } else if chance < MUTATION_CHANCE {
                        // 50% to mutate the col
                        i.col = mutated_coord;
                    }
                    // println!("{} -> {}", pre, i);
                }
                // Check for cancer (two pieces in the same coord)
                // Try to place the genes on a new board and if successful add it to the env, else
                // leave the child to die (cancerous).
                let mut child = Board::new(n);
                for i in child_genes {
                    if child.place(&i).is_err() {
                        continue 'child_production;
                    }
                }

                // Healthy child release to the environment.
                env.push(child);
            }
        }
    }

    /// Solve the problem using a random placement algorithm.
    ///
    /// This implementation implements a heuristic function based on the number of threats queens
    /// pose to each other (includes indirect threats through each other).
    ///
    /// The implementation runs a VERY greedy search which tries to move the most threatened peice
    /// `n` times only if lowers the heuristic. If after a `z` number of tries (defined in
    /// `lower_heuristic`) fails to do so, the program panics.
    ///
    /// This is a homework example of a "heuristic function implementation" not an attempt to solve
    /// the N-Queens.
    pub fn random_solution(n: usize) {
        let mut board = Board::new(n).init_n_queens().unwrap();

        println!(
            "Initial heuristic: {}/{}",
            board.checks_count(),
            board.max_checks()
        );

        const MAX_MOVES: usize = 100000;
        println!("Not printing random moves without a heustiric change");
        for i in 0..MAX_MOVES {
            let pre_move = board.to_string();
            let (pre_h, h, from, to) = Self::lower_heuristic(&mut board)
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

                for i in 0..(pre.len() - 1/* last line is empty */) {
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
}

fn main() {
    const INVALID_MODE_ERROR: &str = "Expected 'random' as the mode";
    let mode = match std::env::args()
        .nth(1)
        .expect(INVALID_MODE_ERROR)
        .to_lowercase()
        .as_str()
    {
        "random" => Mode::Random,
        "genetic" => Mode::Genetic,
        _ => panic!("Invalid mode. {}", INVALID_MODE_ERROR),
    };

    let n = std::env::args()
        .nth(2)
        .expect("Expected an input as the number of Queens (N)")
        .to_owned()
        .parse::<usize>()
        .expect("Invalid number as the input.");

    if n < 4 {
        panic!("Given number must be bigger than 4");
    }

    mode.solve(n)
}
