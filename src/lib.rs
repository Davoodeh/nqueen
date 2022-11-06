//! The library holding implementation free structs of N-Queen.

// TODO replace prints with logs

use rand::prelude::*;
use std::fmt::Display;

const ALREADY_FILLED_POINT_ERROR: &str = "The point is already taken.";

/// A point on a chess board.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl Point {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Create a random point on an `n^2` square.
    pub fn random(n: usize) -> Self {
        let mut rng = thread_rng();
        Self {
            row: rng.gen_range(0..n),
            col: rng.gen_range(0..n),
        }
    }
}

impl TryInto<(i32, i32)> for Point {
    type Error = <i32 as TryFrom<usize>>::Error;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<(i32, i32)> for &Point {
    type Error = <i32 as TryFrom<usize>>::Error;

    fn try_into(self) -> Result<(i32, i32), Self::Error> {
        Ok((self.row.try_into()?, self.col.try_into()?))
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}x{})", self.row + 1, self.col + 1)
    }
}

/// Represents an `n*n` chess board.
#[derive(Debug, Clone)]
pub struct Board {
    /// The coordinates of the queens on the board.
    queens: Vec<Point>,
    /// Size of the chessboard (usually one wants to solve for the same number of queens).
    n: usize,
    /// Caches the relation of checked pieces (gets updated by `mov`, `place` and `capture`).
    check_data: Vec<Vec<Point>>,
    /// Maximum number of checks possible for this configuration.
    max_checks: usize,
}

impl Board {
    pub fn new(n: usize) -> Self {
        Self {
            queens: vec![],
            n,
            check_data: vec![],
            max_checks: 0,
        }
    }

    /// Place some number of `queens` randomly on the board.
    // TODO Optimize the large loop.
    pub fn init_queens(mut self, queens: usize) -> Result<Self, &'static str> {
        let mut placed_queens = 0;
        const MAX_TRIES: usize = 100000;
        for _ in 0..MAX_TRIES {
            if placed_queens == queens {
                println!("Queens: {}", self.queens_display());
                println!("{}", self);
                return Ok(self);
            }
            if self.place(&Point::random(self.n)).is_ok() {
                placed_queens += 1;
            }
        }

        Err("Could not place more queens on the board (is it filled?)")
    }

    /// Place N queens on the board randomly.
    pub fn init_n_queens(self) -> Result<Self, &'static str> {
        let n = self.n;
        self.init_queens(n)
    }

    /// A getter for the check data.
    pub fn check_data(&self) -> &Vec<Vec<Point>> {
        &self.check_data
    }

    /// A getter for the max checks.
    pub fn max_checks(&self) -> usize {
        self.max_checks
    }

    /// Place a Queen on a given point.
    pub fn place(&mut self, point: &Point) -> Result<(), &'static str> {
        if self.queens.contains(point) {
            Err(ALREADY_FILLED_POINT_ERROR)
        } else {
            self.queens.push(point.clone());
            self.update_check_data();
            Ok(())
        }
    }

    /// Removes a Queen from the game.
    pub fn capture(&mut self, point: &Point) {
        self.queens.remove(self.index_of(point).unwrap());
        self.update_check_data();
    }

    /// Move a Queen to another position.
    pub fn mov(&mut self, from: &Point, to: &Point) -> Result<(), &'static str> {
        if self.queens.contains(to) {
            Err(ALREADY_FILLED_POINT_ERROR)
        } else {
            self.queens.push(to.clone());
            self.queens.remove(self.index_of(from).unwrap());
            self.update_check_data();
            Ok(())
        }
    }

    /// Are two of the Queens checking (threatening) each other (counts colat or going thro others).
    pub fn checking(queen1: &Point, queen2: &Point) -> bool {
        if queen1 == queen2 {
            return false;
        }
        let (x1, y1): (i32, i32) = queen1.try_into().unwrap();
        let (x2, y2): (i32, i32) = queen2.try_into().unwrap();
        x1 == x2 || y1 == y2 || (x1 - x2).abs() == (y1 - y2).abs() /* diagonal */
    }

    /// Update the list of all the Queens that are checking each other.
    ///
    /// If this is used as the heuristic, the furthest away from the answer is the number of edges
    /// on a Komplete graph: `(self.queens.len() * self.queens.len().saturating_sub(1)) / 2`.
    ///
    /// This function counts each threat 2 times, once from the point of view
    fn update_check_data(&mut self) {
        let mut v = Vec::<Vec<Point>>::new();
        // For each unique relation check if two are checking, if yes add them to the list.
        let n = self.queens.len();
        for i in 0..n {
            let mut threats = Vec::<Point>::new();
            for j in 0..n {
                let queen1 = &self.queens[i];
                let queen2 = &self.queens[j];
                if Self::checking(queen1, queen2) {
                    threats.push(queen2.clone());
                }
            }
            v.push(threats);
        }
        self.check_data = v;
        self.max_checks = (self.queens.len() * self.queens.len().saturating_sub(1)) / 2;
    }

    pub fn random_point(&self) -> Point {
        Point::random(self.n)
    }

    /// Return the index of the queen which is under the most threat.
    ///
    /// Returns nothing if there is no queen on the board.
    pub fn most_checked(&self) -> Option<usize> {
        // take out the original index and the data to sort them and be able to track the movements
        let mut check_data = self
            .check_data
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.clone()))
            .collect::<Vec<(usize, Vec<Point>)>>();
        // sort by the number of threats
        check_data.sort_by(|a, b| a.1.len().partial_cmp(&b.1.len()).unwrap());
        match check_data.last() {
            Some(v) => Some(v.1.len()),
            None => None,
        }
    }

    /// Move the most threatened queen to another place.
    ///
    /// Returns the source and destination.
    ///
    /// # Caveats
    /// - Pancis if called on a board with no queens.
    /// - Loops forever if the board is filled as it places the queen randomly.
    pub fn move_most_checked(&mut self) -> (Point, Point) {
        let most_checked_index = self
            .most_checked()
            .expect("Place a queen on the board before trying to move the most checked");
        let src = self.queens[most_checked_index].clone();
        let mut dest = self.random_point();
        loop {
            if self.mov(&src, &dest).is_ok() {
                break;
            }
            dest = self.random_point();
        }
        (src, dest)
    }

    /// Search for a queen at a certain position.
    pub fn index_of(&self, point: &Point) -> Option<usize> {
        self.queens.iter().position(|p| *p == *point)
    }

    /// The sum of all the checks and threats that queens make.
    ///
    /// This can be a good heuristic function to estimate the distance to the ideal solution.
    pub fn checks_count(&self) -> usize {
        self.check_data().iter().map(|i| i.len()).sum::<usize>() / 2
    }

    pub fn queens_display(&self) -> String {
        let mut s = String::new();
        const SEP: &str = ", ";
        let n = self.queens.len();
        s += "[";
        for i in 0..n {
            s += &self.queens[i].to_string();
            if i != (n - 1) {
                s += SEP;
            }
        }
        s += "]";
        s
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.n;

        const FILLER: usize = 3; // An odd number! (count of empty spaces between each piece)

        let div = |filler: &str, sep: &str| {
            (filler.repeat(FILLER) + sep).repeat(n - 1) + &filler.repeat(FILLER)
        };
        // empty spaces enough for filling a around a symbol
        let fill = &" ".repeat(FILLER);
        let half_fill = &" ".repeat(FILLER / 2);
        let filled = |i: &str| format!("{0}{1}{0}", half_fill, i);

        let col_counter = &filled(&{
            let mut s = String::new();
            for i in 1..=n {
                s += " "; // for col separator bars |
                s += &filled(&(i % 10).to_string());
            }
            s += half_fill;
            s += "  \n";
            s
        });

        write!(f, " {}", col_counter)?;
        write!(f, "{0}╔{1}╗{0} \n", half_fill, div("═", "╦"))?;
        for i in 0..n {
            let mut row = String::new();
            for j in 0..n {
                match self.index_of(&Point::new(i, j)) {
                    Some(i) => row += &filled(&self.check_data[i].len().to_string()),
                    None => row += fill,
                }

                if j != (n - 1) {
                    row += "│";
                }
            }

            write!(f, "{0} ║{1}║ {0}\n", (i + 1) % 10, row)?;
            if i != (n - 1) {
                write!(f, "  ╠{}╣  \n", div("─", "┼"))?;
            }
        }
        write!(f, "{0} ╚{1}╝ {0}\n", half_fill, div("═", "╩"))?;
        write!(f, " {}", col_counter)?;
        Ok(())
    }
}
