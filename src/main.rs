use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::{cmp::Reverse, collections::VecDeque, ops::Range, usize};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CellState {
    Dead,
    Alive,
    Undetermined,
}

// Define the fixed width and height for the 2D array
const SIZE: usize = 20;
const DIM: i32 = SIZE as i32;

// Create a type alias for the 2D array
type LifeGrid = [[CellState; SIZE]; SIZE];

fn _adjoin(str_a: String, str_b: String, buffer_amt: usize, buffer_char: char) -> String {
    let a_lines: Vec<String> = str_a.split('\n').map(String::from).collect();
    let b_lines: Vec<String> = str_b.split('\n').map(String::from).collect();
    let a_maxlen = a_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let max_lines = a_lines.len().max(b_lines.len());
    let joined_lines: Vec<String> = (0..max_lines)
        .map(|i| {
            format!(
                "{}{}",
                (if i < a_lines.len() {
                    format!(
                        "{}{}",
                        a_lines[i],
                        if i < b_lines.len() {
                            String::from(buffer_char)
                                .repeat((a_maxlen + buffer_amt) - a_lines[i].len())
                        } else {
                            String::from("")
                        }
                    )
                } else {
                    String::from(buffer_char).repeat(buffer_amt)
                }),
                (if i < b_lines.len() {
                    b_lines[i].clone()
                } else {
                    String::from("")
                })
            )
        })
        .collect();
    return joined_lines.join("\n");
}

fn adjoin(str_a: String, str_b: String) -> String {
    return _adjoin(str_a, str_b, 1, ' ');
}

fn to_str(grid: &LifeGrid) -> String {
    let mut grid_str = format!("+{}+\n", "-".repeat(SIZE * 2 + 1));
    for row in grid.iter() {
        grid_str.push('|');
        grid_str.push(' ');
        for cell in row.iter() {
            let symbol = match cell {
                CellState::Dead => ' ',
                CellState::Alive => '#',
                CellState::Undetermined => '~',
            };
            grid_str.push(symbol);
            grid_str.push(' ');
        }
        grid_str.push_str("|\n");
    }
    grid_str.push_str(format!("+{}+", "-".repeat(SIZE * 2 + 1)).as_str());
    grid_str
}

fn to_str_marked(grid: &LifeGrid, mx: usize, my: usize) -> String {
    let mut grid_str = format!("+{}+\n", "-".repeat(SIZE * 2 + 1));
    for (x, row) in grid.iter().enumerate() {
        grid_str.push('|');
        grid_str.push(' ');
        for (y, cell) in row.iter().enumerate() {
            grid_str.push(if x == mx && y == my {
                'O'
            } else {
                match cell {
                    CellState::Dead => ' ',
                    CellState::Alive => '#',
                    CellState::Undetermined => '~',
                }
            });
            grid_str.push(' ');
        }
        grid_str.push_str("|\n");
    }
    grid_str.push_str(format!("+{}+", "-".repeat(SIZE * 2 + 1)).as_str());
    grid_str
}

fn print_life_grid(grid: &LifeGrid) {
    print_life_grids(&[*grid]);
}

fn print_life_grids(grids: &[LifeGrid]) {
    let mut out_str: String = to_str(&grids[0]);
    for grid in &grids[1..] {
        out_str = adjoin(out_str, to_str(grid));
    }
    println!("{}", out_str);
}

fn print_grid_grid(grids: &Vec<LifeGrid>, width: usize) {
    for i in (0..grids.len()).step_by(width) {
        print_life_grids(&grids[i..((i + width).min(grids.len()))]);
    }
}

fn count_adjacent_cells(grid: &LifeGrid, x: &usize, y: &usize, wrap: bool) -> Range<i32> {
    let (mut min_total, mut max_total) = (0, 0);
    for dx in -1..=1 {
        for dy in -1..=1 {
            if !(dx == 0 && dy == 0) {
                let (nx, ny): (i32, i32) = (*x as i32 + dx, *y as i32 + dy);
                let cell_value = if wrap {
                    grid[(nx as usize + SIZE) % SIZE][(ny as usize + SIZE) % SIZE]
                } else {
                    if nx < 0 || ny < 0 || nx >= DIM || ny >= DIM {
                        CellState::Dead
                    } else {
                        grid[nx as usize][ny as usize]
                    }
                };
                let (min_add, max_add) = match cell_value {
                    CellState::Alive => (1, 1),
                    CellState::Undetermined => (0, 1),
                    CellState::Dead => (0, 0),
                };
                min_total += min_add;
                max_total += max_add;
            }
        }
    }
    min_total..max_total
}

fn simulate_grid(grid: &LifeGrid, wrap: bool) -> LifeGrid {
    let mut next_grid = [[CellState::Undetermined; SIZE]; SIZE];
    for x in 0..SIZE {
        for y in 0..SIZE {
            let count_range = count_adjacent_cells(grid, &x, &y, wrap);
            // println!("{}", adjoin(to_str(&grid), adjoin(to_str(&next_grid), to_str_marked(&next_grid, x, y))));
            // println!("");
            next_grid[x][y] = match grid[x][y] {
                CellState::Alive => {
                    if superset(&(2..3), &count_range) {
                        CellState::Alive
                    } else if superset(&(0..1), &count_range) || superset(&(4..8), &count_range) {
                        CellState::Dead
                    } else {
                        CellState::Undetermined
                    }
                }
                CellState::Dead => {
                    if superset(&(3..3), &count_range) {
                        CellState::Alive
                    } else if !overlap(&(3..3), &count_range) {
                        CellState::Dead
                    } else {
                        CellState::Undetermined
                    }
                }
                CellState::Undetermined => CellState::Undetermined,
            };
        }
    }
    next_grid
}

fn overlap(a: &Range<i32>, b: &Range<i32>) -> bool {
    !(a.end < b.start || b.end < a.start)
}

fn superset(a: &Range<i32>, b: &Range<i32>) -> bool {
    a.start <= b.start && a.end >= b.end
}

fn is_consistent(grid_a: &LifeGrid, grid_b: &LifeGrid) -> bool {
    if *grid_a == *grid_b {
        return true;
    } else {
        for (x, row) in grid_a.iter().enumerate() {
            for (y, item) in row.iter().enumerate() {
                let other_item = grid_b[x][y];
                if *item == CellState::Alive && other_item == CellState::Dead
                    || *item == CellState::Dead && other_item == CellState::Alive
                {
                    return false;
                }
            }
        }
        return true;
    }
}

fn explore_possible_prior_grids(
    present_grid: &LifeGrid,
    skip_lonely_cells: bool,
    wrap: bool,
    yield_amount: Option<usize>,
) -> Vec<LifeGrid> {
    let mut stack: Vec<(LifeGrid, i32, i32)> = Vec::new();
    stack.push(([[CellState::Undetermined; SIZE]; SIZE], 0, 0));
    let mut results = Vec::new();
    let mut explored_grids = 0;
    while let Some((possible_past_grid, x, y)) = stack.pop() {
        explored_grids += 1;
        let simulated_present_grid = simulate_grid(&possible_past_grid, wrap);
        // print_grid_grid(&stack.iter().map(|(g,_,_)| g).collect(), 20);
        // println!("{}", adjoin(to_str(&present_grid), adjoin(to_str(&simulated_present_grid), adjoin(to_str(&possible_past_grid), to_str_marked(&possible_past_grid, x as usize, y as usize)))));
        // println!("");
        if !is_consistent(&simulated_present_grid, present_grid) {
            continue;
        }
        if y >= DIM {
            results.push(possible_past_grid);
            let num_results = results.len();
            if yield_amount.map(|n| num_results >= n).unwrap_or(false) {
                return results;
            }
            if results.len() % 1000 == 0 {
                print_life_grid(&possible_past_grid);
                println!("{num_results} results, {explored_grids} total explored grids");
            }
        } else {
            let current_cell_present_state = present_grid[x as usize][y as usize];
            let living_range =
                count_adjacent_cells(&possible_past_grid, &(x as usize), &(y as usize), wrap);
            /*
             * If the current cell is alive:
             *  - it could only have been alive last step if it had either 2 or 3 living neighbors
             *  - it could only have been dead last step if it had exactly 3 living neighbors
             * If the current cell is dead:
             *  - it could only have been alive last step if it has 0, 1, 4, or greater neighbors
             *  - it could only have been dead last step if it had 0, 1, 2, 4, or greater neighbors
             * also, don't check lonely cells to see if they could have been alive in the previous state
             */

            let check = match current_cell_present_state {
                CellState::Alive => [
                    overlap(&living_range, &(2..3)),
                    overlap(&living_range, &(3..3)),
                ],
                CellState::Dead => [
                    (overlap(&living_range, &(0..1)) || overlap(&living_range, &(4..8)))
                        && (!skip_lonely_cells
                            || count_adjacent_cells(
                                &present_grid,
                                &(x as usize),
                                &(y as usize),
                                wrap,
                            )
                            .end > 0),
                    true,
                ],
                CellState::Undetermined => [true, true],
            };

            let total_index = x + y * DIM;
            let next_index = total_index + 1;
            let (next_x, next_y) = (next_index % DIM, next_index / DIM);
            for i in 0..2 {
                if check[i] {
                    let mut new_grid = possible_past_grid.clone();
                    new_grid[x as usize][y as usize] = [CellState::Alive, CellState::Dead][i];
                    stack.push((new_grid, next_x, next_y));
                }
            }
        }
    }
    let total_results = results.len();
    println!("{total_results} total results");
    results
}

fn explore_possible_prior_grids_parallel(
    present_grid: &LifeGrid,
    skip_lonely_cells: bool,
    wrap: bool,
    shy_from_edges: bool,
) -> Vec<LifeGrid> {
    let mut stack: Vec<LifeGrid> = Vec::new();
    stack.push([[CellState::Undetermined; SIZE]; SIZE]);
    let mut total_index: usize = 0;
    let pbar = ProgressBar::new((SIZE * SIZE) as u64);
    pbar.set_prefix("Evaluating");
    while total_index < SIZE * SIZE {
        let (x, y) = (total_index % SIZE, total_index / SIZE);
        let at_edge = x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1;
        stack = stack
            .par_iter()
            .map(|possible_past_grid| {
                let simulated_present_grid = simulate_grid(&possible_past_grid, wrap);
                if !is_consistent(&simulated_present_grid, present_grid) {
                    vec![]
                } else {
                    let current_cell_present_state = present_grid[x][y];
                    let living_range = count_adjacent_cells(&possible_past_grid, &x, &y, wrap);

                    let check = match current_cell_present_state {
                        CellState::Alive => [
                            overlap(&living_range, &(2..3)),
                            overlap(&living_range, &(3..3)),
                        ],
                        CellState::Dead => [
                            (overlap(&living_range, &(0..1)) || overlap(&living_range, &(4..8)))
                                && (!skip_lonely_cells
                                    || count_adjacent_cells(&present_grid, &x, &y, wrap).end > 0),
                            true,
                        ],
                        CellState::Undetermined => [true, true],
                    };

                    (0..2)
                        .filter(|i| {
                            if *i == 0 && shy_from_edges && at_edge {
                                false
                            } else {
                                check[*i as usize]
                            }
                        })
                        .map(|i| {
                            let mut new_grid = possible_past_grid.clone();
                            new_grid[x as usize][y as usize] =
                                [CellState::Alive, CellState::Dead][i];
                            new_grid
                        })
                        .collect()
                }
            })
            .flatten()
            .collect();
        total_index += 1;
        let stack_len = stack.len();
        let max_index = DIM * DIM;
        pbar.inc(1);
        pbar.set_message(format!("{stack_len}\t{total_index}/{max_index}"));
    }

    let total_results = stack.len();
    pbar.finish_with_message(format!("{total_results} total results"));
    stack
}

fn num_cells(grid: &LifeGrid) -> usize {
    grid.iter()
        .map(|r| r.iter().filter(|cell| **cell == CellState::Alive).count())
        .sum::<usize>()
}

fn shape_radius(grid: &LifeGrid) -> usize {
    let row_spans: Vec<(Option<usize>, Option<usize>)> = grid
        .iter()
        .map(|row| {
            (
                row.iter()
                    .enumerate()
                    .find(|(_, cell)| **cell == CellState::Alive)
                    .map(|(n, _)| n),
                row.iter()
                    .enumerate()
                    .rfind(|(_, cell)| **cell == CellState::Alive)
                    .map(|(n, _)| n),
            )
        }).collect();
    let width = match (
        row_spans.iter().map(|(l, _)| l).filter_map(|x|*x).min(),
        row_spans.iter().map(|(_, r)| r).filter_map(|x|*x).max(),
    ) {
        (Some(left), Some(right)) => right - left + 1,
        _ => 0
    };
    let rows_with_cells: Vec<(usize, bool)> = grid
        .iter()
        .map(|row| row.iter().any(|cell| *cell == CellState::Alive))
        .enumerate()
        .collect();
    let height = match (
        rows_with_cells
            .iter()
            .find(|(_, flag)| *flag)
            .map(|(n, _)| n),
        rows_with_cells
            .iter()
            .rfind(|(_, flag)| *flag)
            .map(|(n, _)| n),
    ) {
        (Some(top), Some(bottom)) => bottom - top + 1,
        _ => 0,
    };
    width * height
}

fn sim_backwards(starting_grid: &LifeGrid, steps: usize) -> Option<LifeGrid> {
    let mut state_stack: Vec<Vec<LifeGrid>> = vec![vec![*starting_grid]];
    while state_stack.len() < steps {
        let step = state_stack.len();
        match state_stack.pop() {
            None => {
                println!("No more steps left");
                return None;
            }
            Some(mut grid_set) => {
                let mut explored_all = true;
                while !grid_set.is_empty() {
                    let grid = grid_set.pop().unwrap();
                    print_life_grid(&grid);
                    println!("Simulating step {step}");
                    let mut results =
                        explore_possible_prior_grids_parallel(&grid, true, false, true);
                    if !results.is_empty() {
                        results.sort_by_cached_key(|grid| Reverse(shape_radius(&grid)));
                        // print_grid_grid(&results, 5);
                        state_stack.push(grid_set);
                        state_stack.push(results);
                        explored_all = false;
                        break;
                    } else {
                        println!("No paths from the current grid!");
                    }
                }
                if explored_all {
                    println!("No grids left at current step");
                }
            }
        }
    }
    Some(state_stack.pop().unwrap().pop().unwrap())
}

fn main() {
    let mut grid: LifeGrid = [[CellState::Dead; SIZE]; SIZE];
    let glider = &[(0, 2), (1, 0), (1, 2), (2, 1), (2, 2)];
    let glider_2 = &[(0, 0), (1, 1), (1, 2), (2, 0), (2, 1)];
    let oscillator = &[(0, 0), (0, 1), (0, 2)];
    for (x, y) in glider {
        grid[*x + 15][*y + 15] = CellState::Alive
    }
    let back_sim = sim_backwards(&grid, 35).unwrap();
    print_life_grid(&back_sim);
    // let grids = explore_possible_prior_grids_parallel(&grid, true, false);
    // print_grid_grid(&grids, 100/DIM as usize);
}
