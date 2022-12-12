//! Solves N-Queens problem with different basic algorithms.
//!
//! See `Mode` enum for the various algorithms implmemented.

#[macro_use]
extern crate clap;

use clap::{arg, command, Parser};
use rand::prelude::*;

use nqueen::{Board, Point};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: ModeCommands,
}

/// The possible modes/algorithms of the program.
///
/// NOTE I'm well aware that one should not write everything in a single function. But I saw fit to
/// implement ONE and ONLY ONE `*-solution` method for each variant. This is a deliberate choice.
#[derive(Subcommand, Debug)]
enum ModeCommands {
    /// Use a random placement (heuristic based) algorithm
    Random {
        /// The size of the board and number of queens
        #[arg(value_parser = clap::value_parser!(u16).range(4..))]
        n: u16,
    },
    /// Use genetic algorithm
    Genetic {
        /// The size of the board and number of queens
        #[arg(value_parser = clap::value_parser!(u16).range(4..))]
        n: u16,
        /// The initial population of the boards
        #[arg(short, long, value_name = "POPULATION", default_value_t = 30)]
        population: usize,
        /// The number of parents that combine into a single child
        #[arg(short = 'r', long, value_name = "PARENTS", default_value_t = 2)]
        parents: usize,
        /// The maximum number of suvivors moved to the next generation
        #[arg(short, long, value_name = "SURVIVORS", default_value_t = 6)]
        survivors: usize,
        /// The mutation percentage for each single gene
        #[arg(short, long, value_name = "SURVIVORS", default_value_t = 5)]
        mutation_chance: usize,
        /// The number of moves in each generation
        #[arg(
            short = 'd',
            long,
            value_name = "MOVES_IN_GENERATION",
            default_value_t = 3
        )]
        moves_in_generation: usize,
        /// The maximum number of generations
        #[arg(short, long, value_name = "GENERATIONS", default_value_t = 1000)]
        generations: usize,
    },
}

impl ModeCommands {
    /// Solve the selected mode.
    pub fn solve(&self) {
        match self {
            Self::Random { .. } => self.random_solution(),
            Self::Genetic { .. } => self.genetic_solution(),
        }
    }

    /// Solve the problem using the genetics algorithm.
    fn genetic_solution(&self) {
        // TODO update to let-else when the new Rust is out
        let (n, population, parents, survivors, mutation_chance, generations, moves_in_generation) =
            match *self {
                Self::Genetic {
                    population,
                    parents,
                    survivors,
                    mutation_chance,
                    generations,
                    moves_in_generation,
                    ..
                } => (
                    self.n(),
                    population,
                    parents,
                    survivors,
                    mutation_chance,
                    generations,
                    moves_in_generation,
                ),
                _ => unreachable!("Invalid variant called the genetic solution"),
            };

        assert!(
            parents < n,
            "The chess board size is smaller than the number of parents.\
             To keep the code simple, this is not supported."
        );

        let mut rng = thread_rng();

        // Holds the boards.
        let mut env = vec![Board::new(n); population]
            .into_iter()
            .map(|i| i.init_n_queens().unwrap())
            .collect::<Vec<Board>>();
        // Create the primitive/initial boards, the natives of the env.
        // Print all the heuristics of the boards in the env
        let all_heuristics =
            |env: &Vec<Board>| env.iter().map(|i| i.checks_count()).collect::<Vec<usize>>();

        println!(
            "Environment details:\n\
             - initial population: {population}\n\
             - each generation's max survivors: {survivors}\n\
             - number of parents required for a child: {parents}\n\
             - mutation chance: {mutation_chance}%\n\
             - moves before a generation dies out: {moves_in_generation}\n\
             - maximum generations: {generations}\n\
            "
        );

        'time: for generation in 0..generations {
            println!("Generation #{}", generation);
            println!("This generation's heuristics: {:?}", all_heuristics(&env));

            // Let them live their lives
            for board in env.iter_mut() {
                for _ in 0..moves_in_generation {
                    let _ = Self::lower_heuristic(board);
                }
            }

            // Sort by fitness
            env.sort_by(|a, b| a.checks_count().cmp(&b.checks_count()));
            println!(
                "This generation's heuristics after {} moves: {:?}",
                moves_in_generation,
                all_heuristics(&env)
            );

            // Pick this generation of survivors and check for the fittest or continue.
            if env.len() < survivors {
                panic!("Everybody died!");
            }
            let survivors_vec = &env[0..survivors].to_vec();
            let survivors_heuristics = all_heuristics(&survivors_vec);
            println!(
                "The {} survivors of this generation are: {:?}",
                survivors, survivors_heuristics,
            );
            if survivors_heuristics[0] == 0 {
                println!("The fittest was found!");
                println!("{}", survivors_vec[0]);
                break 'time;
            }

            env = vec![];
            // Make children from the survivors
            'child_production: for _parents in 0..(population / parents) {
                // println!("Managing the couple #{}", _parents);
                // Choose some parents to make a child from them
                let mut parents_vec = Vec::<Board>::with_capacity(parents);
                for _ in 0..parents {
                    // NOTE Since this is not important, we leave the chance for a board to have
                    // children from itself.
                    let randomly_picked_parent = survivors_vec[rng.gen_range(0..survivors)].clone();
                    parents_vec.push(randomly_picked_parent);
                }

                // Create the child from their stats and distribute the information/genes equally.
                // In this example implementation, the gene split rate is uniform meaning all
                // parents pass equal amount of genes to their children.
                let mut child_genes = Vec::<Point>::with_capacity(n);
                let gene_portions = n / parents; // number of genes from each parent
                let last_parent_extra_passing = n % parents; // leftover genes for the last parent
                for i in 0..parents {
                    // println!("Subject parent #{}: {}", i, parents[i].queens_display());
                    let mut extra_genes = 0;
                    // As for the last parent, give all the remaining genes to the child (may be
                    // more than others).
                    if i == parents - 1 {
                        extra_genes = last_parent_extra_passing;
                    }
                    for j in 0..(gene_portions + extra_genes) {
                        // Pick the first n/PARENTS genes from the first parent then the next till
                        // one is left
                        let gene = parents_vec[i].queens()[(i * gene_portions) + j].clone();
                        // println!("Inheriting {} from parent #{}", gene, i);
                        child_genes.push(gene);
                    }
                }

                // Mutate the child genes (move the pieces randomly).
                // If two pieces collide, mark the board as invalid/cancerous/high-cost (removes it
                // from the next generation).
                for i in child_genes.iter_mut() {
                    let chance = rng.gen_range(0..100);
                    let mutated_coord = rng.gen_range(0..n);
                    // println!(
                    //     "Mutation: {}>={} ; Mutated Coord: {}",
                    //     mutation_chance,
                    //     chance,
                    //     (mutated_coord + 1) // Point is +1 in its diplay
                    // );
                    if chance < (mutation_chance / 2) {
                        // 50% to mutate the row
                        i.row = mutated_coord;
                    } else if chance < mutation_chance {
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
    pub fn random_solution(&self) {
        let n = self.n();

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

    fn n(&self) -> usize {
        match self {
            Self::Genetic { n, .. } => *n as usize,
            Self::Random { n, .. } => *n as usize,
        }
    }
}

fn main() {
    let cli = Cli::parse().mode;
    cli.solve();
}
