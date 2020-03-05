pub struct Grid {
    size: (usize, usize),
    cells: Vec<Cell>,
}

#[derive(Clone)]
pub struct Cell {
    content: String,
    alignment: Alignment,
    border: Border,
    ident: Ident,
    span_row: usize,
}

#[derive(Clone)]
struct Border {
    top: String,
    bottom: String,
    left: String,
    right: String,
    corner: String,
}

#[derive(Clone)]
struct Ident {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize,
}

#[derive(Clone, Copy)]
pub enum Alignment {
    Center,
    Left,
    Right,
}

impl Grid {
    pub fn new(rows: usize, columns: usize) -> Self {
        Grid {
            size: (rows, columns),
            cells: vec![Cell::new(); rows * columns],
        }
    }

    pub fn cell(&mut self, i: usize, j: usize) -> &mut Cell {
        let index = self.count_columns() * i + j;
        self.cells.get_mut(index).unwrap()
    }

    pub fn count_rows(&self) -> usize {
        self.size.0
    }

    pub fn count_columns(&self) -> usize {
        self.size.1
    }
}

fn columns_from_rows<'a>(rows: &'a [&'a [Cell]]) -> Vec<Vec<&'a Cell>> {
    let count_columns = rows[0].len();
    let count_rows = rows.len();
    (0..count_columns)
        .map(|column| {
            (0..count_rows)
                .map(|row| &rows[row][column])
                .collect::<Vec<_>>()
        })
        .collect()
}

fn rows<T>(slice: &[T], count_rows: usize, count_columns: usize) -> Vec<&[T]> {
    (0..count_rows)
        .map(|row_index| {
            let row_start = count_columns * row_index;
            &slice[row_start..row_start + count_columns]
        })
        .collect()
}

fn remove_covered_cells(rows: &[&[Cell]]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|row| {
            row.iter()
                .scan(0, |skip, cell| {
                    if *skip > 0 {
                        *skip -= 1;
                        Some(None)
                    } else {
                        *skip = cell.span_row;
                        Some(Some(cell.clone()))
                    }
                })
                .flatten()
                .collect::<Vec<Cell>>()
        })
        .collect()
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = rows(&self.cells, self.size.0, self.size.1);

        let rows_weight = rows
            .iter()
            .map(|row| row.iter().map(|c| c.weight()).collect())
            .collect::<Vec<Vec<usize>>>();

        println!("0000 {:?}", rows_weight);

        let columns_weight = rows
            .iter()
            .map(|r| r.iter().map(|c| c.weight()).sum::<usize>())
            .max()
            .map_or(0, |m| m);

        println!("weight {}", columns_weight);

        let portions = row_portions_blocks(&rows);
        println!("portions {:?}", portions);
        // let rows = remove_covered_cells(&rows);
        let size = cell_size(&rows, &portions, columns_weight);

        println!("sizes {:?}", size);

        let grid = rows
            .into_iter()
            .enumerate()
            .map(move |(row_index, row)| {
                row.into_iter()
                    .enumerate()
                    .flat_map(|(column_index, cell)| {
                        if size[row_index][column_index].0 == 0 {
                            return None;
                        }

                        let mut formatter = CellFormatter::new(cell)
                            .weight(size[row_index][column_index].0)
                            .height(size[row_index][column_index].1)
                            .boxed();

                        if column_index != 0 {
                            formatter = formatter.un_left().un_left_connection();
                        }

                        if row_index != 0 {
                            formatter = formatter.un_top();
                        }

                        Some(formatter)
                    })
                    .collect()
            })
            .collect::<Vec<Vec<CellFormatter>>>();

        let grid = adjust(&grid);

        for row in grid {
            let row = row.iter().map(|f| f.format()).collect::<Vec<String>>();

            writeln!(f, "{}", concat_row(&row))?;
        }

        Ok(())
    }
}

fn cell_size(
    rows: &[&[Cell]],
    portions: &[Vec<f32>],
    mut weight: usize,
) -> Vec<Vec<(usize, usize)>> {
    let rows_height = rows
        .iter()
        .map(|row| row.iter().map(|cell| cell.height()).max().unwrap())
        .collect::<Vec<usize>>();

    let mut sizes = measure_cell_size(rows, portions, &rows_height, weight);
    while !is_rows_size(&sizes, weight) {
        weight += 1;
        sizes = measure_cell_size(rows, portions, &rows_height, weight);
    }

    println!("{}", is_rows_size(&sizes, weight));
    println!("{:?}", sizes);

    sizes
}

fn row_cell_weight(row: &[Cell]) -> usize {
    row_weight(
        &row.iter()
            .map(|cell| cell.weight() + cell.ident.left + cell.ident.right)
            .collect::<Vec<usize>>(),
    )
}

fn is_rows_size(rows: &[Vec<(usize, usize)>], weight: usize) -> bool {
    let row_weights = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|(w, ..)| *w)
                .filter(|w| *w > 0)
                .collect::<Vec<usize>>()
        })
        .map(|weights| row_weight(&weights))
        .collect::<Vec<usize>>();
    row_weights.iter().min() == row_weights.iter().max()
}

fn measure_cell_size(
    rows: &[&[Cell]],
    portions: &[Vec<f32>],
    rows_height: &[usize],
    weight: usize,
) -> Vec<Vec<(usize, usize)>> {
    rows.iter()
        .enumerate()
        .map(|(row_index, row)| {
            (0..row.len())
                .map(|cell_index| {
                    {
                        println!(
                            "portion; {} w; {} res: {}",
                            portions[row_index][cell_index],
                            weight,
                            weight as f32 * portions[row_index][cell_index]
                        )
                    };
                    let portion = portions[row_index][cell_index];
                    (
                        (weight as f32 * portion).floor() as usize,
                        rows_height[row_index],
                    )
                })
                .collect()
        })
        .collect()
}

fn row_weight(row: &[usize]) -> usize {
    row.iter().sum::<usize>() + row.len() - 1
}

fn row_portions(rows: &[&[Cell]]) -> Vec<Vec<f32>> {
    rows.iter()
        .map(|row| {
            row.iter()
                .scan(0, |skip, cell| {
                    if *skip > 0 {
                        *skip -= 1;
                        Some(0.0)
                    } else {
                        let span = cell.span_row;
                        *skip = span;
                        Some((1.0 + span as f32) / row.len() as f32)
                    }
                })
                .collect()
        })
        .collect()
}

fn row_portions_honest(rows: &[&[Cell]]) -> Vec<Vec<f32>> {
    rows.iter()
        .map(|row| {
            let row_weight = row
                .iter()
                .map(|cell| cell.weight() + cell.ident.left + cell.ident.right)
                .sum::<usize>();
            row.iter()
                .scan(0, |skip, cell| {
                    if *skip > 0 {
                        *skip -= 1;
                        Some(0.0)
                    } else {
                        let span = cell.span_row;
                        *skip = span;
                        Some(
                            ((cell.weight() + cell.ident.left + cell.ident.right) as f32)
                                / row_weight as f32,
                        )
                    }
                })
                .collect()
        })
        .collect()
}

fn row_portions_blocks(rows: &[&[Cell]]) -> Vec<Vec<f32>> {
    let mut blocks: Vec<Vec<&[Cell]>> = rows
        .iter()
        .map(|r| r.iter().map(|c| c.span_row).collect::<Vec<usize>>())
        .zip(0..rows.len())
        .fold(
            Vec::new(),
            |mut blocks: Vec<Vec<(Vec<usize>, usize)>>, (spans, row_index)| {
                match blocks.last_mut() {
                    Some(ref mut block) => match block.last() {
                        Some(block_row) if &block_row.0 == &spans => block.push((spans, row_index)),
                        _ => blocks.push(vec![(spans, row_index)]),
                    },
                    None => {
                        blocks.push(vec![(spans, row_index)]);
                    }
                }
                blocks
            },
        )
        .iter()
        .map(|block| {
            block
                .iter()
                .map(|(_, row_index)| rows[*row_index])
                .collect()
        })
        .collect();

    blocks
        .iter()
        .flat_map(|block| {
            let columns_weight = block
                .iter()
                .map(|column| column.iter().map(|cell| cell.weight()).collect())
                .collect::<Vec<Vec<usize>>>();

            let weight = block
                .iter()
                .map(|r| r.iter().map(|c| c.weight()).sum::<usize>())
                .max()
                .map_or(0, |m| m);

            println!("ww {}", weight);

            let row_portions = block
                .iter()
                .enumerate()
                .map(|(row_index, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(cell_index, c)| {
                            columns_weight[row_index][cell_index] as f32 / weight as f32
                        })
                        .collect()
                })
                .collect::<Vec<Vec<f32>>>();

            let max_portions = row_portions.iter().skip(1).fold(
                row_portions[0].clone(),
                |mut max_portions, row| {
                    row.iter().enumerate().for_each(|(index, portion)| {
                        max_portions[index] = f32::max(max_portions[index], *portion)
                    });
                    max_portions
                },
            );

            (0..block.len())
                .map(|_| max_portions.clone())
                .collect::<Vec<Vec<f32>>>()
        })
        .collect()
}

impl Cell {
    fn new() -> Self {
        Cell {
            alignment: Alignment::Center,
            content: String::new(),
            border: Border {
                top: "-".to_owned(),
                bottom: "-".to_owned(),
                left: "|".to_owned(),
                right: "|".to_owned(),
                corner: "+".to_owned(),
            },
            ident: Ident {
                top: 0,
                bottom: 0,
                left: 0,
                right: 0,
            },
            span_row: 0,
        }
    }

    pub fn set_content(&mut self, s: &str) -> &mut Self {
        self.content = s.to_owned();
        self
    }

    pub fn set_corner(&mut self, s: &str) -> &mut Self {
        self.border.corner = s.to_owned();
        self
    }

    pub fn set_alignment(&mut self, a: Alignment) -> &mut Self {
        self.alignment = a;
        self
    }

    pub fn set_vertical_ident(&mut self, size: usize) -> &mut Self {
        self.ident.top = size;
        self.ident.bottom = size;
        self
    }

    pub fn set_horizontal_ident(&mut self, size: usize) -> &mut Self {
        self.ident.left = size;
        self.ident.right = size;
        self
    }

    pub fn set_row_span(&mut self, size: usize) -> &mut Self {
        self.span_row = size;
        self
    }

    fn height(&self) -> usize {
        self.content.lines().count()
    }

    fn weight(&self) -> usize {
        self.content
            .lines()
            .map(|l| l.len())
            .max()
            .map_or(0, |max| max)
    }
}

#[derive(Clone)]
struct CellFormatter<'a> {
    cell: &'a Cell,
    left: Option<()>,
    right: Option<()>,
    top: Option<()>,
    bottom: Option<()>,
    left_connection: Option<()>,
    right_connection: Option<()>,
    weight: usize,
    height: usize,
}

impl<'a> CellFormatter<'a> {
    fn new(cell: &'a Cell) -> Self {
        CellFormatter {
            cell: cell,
            left: None,
            right: None,
            top: None,
            bottom: None,
            left_connection: None,
            right_connection: None,
            weight: 0,
            height: 0,
        }
    }

    fn un_left(mut self) -> Self {
        self.left = None;
        self
    }

    fn un_left_connection(mut self) -> Self {
        self.left_connection = None;
        self
    }

    fn un_top(mut self) -> Self {
        self.top = None;
        self
    }

    fn un_bottom(mut self) -> Self {
        self.bottom = None;
        self
    }

    fn boxed(mut self) -> Self {
        self.left = Some(());
        self.right = Some(());
        self.top = Some(());
        self.bottom = Some(());
        self.right_connection = Some(());
        self.left_connection = Some(());
        self
    }

    fn weight(mut self, w: usize) -> Self {
        self.weight = w;
        self
    }

    fn full_weight(&self) -> usize {
        self.weight + self.cell.ident.left + self.cell.ident.right
    }

    fn height(mut self, h: usize) -> Self {
        self.height = h;
        self
    }

    fn format(&self) -> String {
        let c = self.cell;
        let weight = if self.weight == 0 {
            c.content
                .lines()
                .map(|l| l.chars().count())
                .max()
                .map_or(0, |max| max)
        } else {
            self.weight
        };

        let mut content = c.content.clone();
        let count_lines = c.content.chars().filter(|&c| c == '\n').count();

        if self.height > count_lines {
            content.push_str(&"\n".repeat(self.height - count_lines))
        }

        content.push_str(&"\n".repeat(c.ident.bottom));
        content.insert_str(0, &"\n".repeat(c.ident.top));

        let left_ident = " ".repeat(c.ident.left);
        let right_ident = " ".repeat(c.ident.right);

        let left_border = self.left.map_or("", |_| &c.border.left);
        let right_border = self.right.map_or("", |_| &c.border.right);

        let mut lines = content
            .lines()
            .map(|l| align(l, c.alignment, weight))
            .map(|l| format!("{}{}{}", left_ident, l, right_ident))
            .map(|l| {
                format!(
                    "{left:}{}{right:}",
                    l,
                    left = left_border,
                    right = right_border,
                )
            })
            .collect::<Vec<String>>();

        let lhs = self.left_connection.map_or("", |_| &c.border.corner);
        let rhs = self.right_connection.map_or("", |_| &c.border.corner);

        let weight = weight + c.ident.left + c.ident.right;

        if self.top.is_some() {
            let line = lhs.to_owned() + &c.border.top.repeat(weight) + rhs;
            lines.insert(0, line);
        }
        if self.bottom.is_some() {
            let line = lhs.to_owned() + &c.border.bottom.repeat(weight) + rhs;
            lines.push(line);
        }

        lines.join("\n")
    }
}

fn adjust<'a>(rows: &[Vec<CellFormatter<'a>>]) -> Vec<Vec<CellFormatter<'a>>> {
    let weight = rows
        .iter()
        .map(|r| r.iter().map(|f| f.full_weight()).sum::<usize>() + r.len() - 1)
        .max()
        .map_or(0, |m| m);

    let w = rows
        .iter()
        .map(|r| r.iter().map(|f| f.full_weight()).collect::<Vec<usize>>())
        .collect::<Vec<_>>();

    println!("iii {:?}", weight);
    println!("zzx {:?}", w);

    rows.iter()
        .map(|r| {
            let row_weight = r.iter().map(|f| f.full_weight()).sum::<usize>();
            let rest_weight = weight - row_weight;
            let squized_space = rest_weight / r.len();

            println!("rw={} rew={} qs={}", row_weight, rest_weight, squized_space);

            r.iter()
                .map(|f| f.clone().weight(f.weight + squized_space))
                .collect()
        })
        .collect()
}

fn align(text: &str, a: Alignment, length: usize) -> String {
    match a {
        Alignment::Center => format!("{: ^1$}", text, length),
        Alignment::Left => format!("{: <1$}", text, length),
        Alignment::Right => format!("{: >1$}", text, length),
    }
}

fn concat_row(row: &[String]) -> String {
    let mut iter = row.iter();
    if let Some(row) = iter.next() {
        let mut row = row.to_owned();
        for c in iter {
            row = concat_lines(&row, c);
        }

        row
    } else {
        "".to_owned()
    }
}

fn concat_lines(a: &str, b: &str) -> String {
    assert_eq!(a.lines().count(), b.lines().count());
    a.lines()
        .zip(b.lines())
        .map(|(a, b)| a.to_owned() + b)
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod grid {
        use super::super::*;

        #[test]
        fn render() {
            let mut grid = Grid::new(2, 2);
            grid.cell(0, 0).set_content("0-0");
            grid.cell(0, 1).set_content("0-1");
            grid.cell(1, 0).set_content("1-0");
            grid.cell(1, 1).set_content("1-1");

            let expected = concat!(
                "+---+---+\n",
                "|0-0|0-1|\n",
                "+---+---+\n",
                "|1-0|1-1|\n",
                "+---+---+\n",
            );

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_multilane() {
            let mut grid = Grid::new(2, 2);
            grid.cell(0, 0).set_content("left\ncell");
            grid.cell(0, 1).set_content("right one");
            grid.cell(1, 0)
                .set_content("the second column got the beginning here");
            grid.cell(1, 1)
                .set_content("and here\nwe\nsee\na\nlong\nstring");

            let expected = concat!(
                "+----------------------------------------+---------+\n",
                "|                  left                  |right one|\n",
                "|                  cell                  |         |\n",
                "+----------------------------------------+---------+\n",
                "|the second column got the beginning here|and here |\n",
                "|                                        |   we    |\n",
                "|                                        |   see   |\n",
                "|                                        |    a    |\n",
                "|                                        |  long   |\n",
                "|                                        | string  |\n",
                "+----------------------------------------+---------+\n",
            );

            let g = grid.to_string();
            assert_eq!(expected, g);
        }

        #[test]
        fn render_one_line() {
            let mut grid = Grid::new(1, 1);
            grid.cell(0, 0).set_content("one line");

            let expected = concat!("+--------+\n", "|one line|\n", "+--------+\n",);

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_not_quadratic() {
            let mut grid = Grid::new(1, 2);
            grid.cell(0, 0).set_content("hello");
            grid.cell(0, 1).set_content("world");

            let expected = concat!("+-----+-----+\n", "|hello|world|\n", "+-----+-----+\n",);

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_empty() {
            let grid = Grid::new(0, 0);

            let expected = "";

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_empty_cell() {
            let mut grid = Grid::new(2, 2);
            grid.cell(0, 0).set_content("0-0");
            grid.cell(0, 1).set_content("");
            grid.cell(1, 0).set_content("1-0");
            grid.cell(1, 1).set_content("1-1");

            let expected = concat!(
                "+---+---+\n",
                "|0-0|   |\n",
                "+---+---+\n",
                "|1-0|1-1|\n",
                "+---+---+\n",
            );

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_row_span() {
            let mut grid = Grid::new(2, 2);
            grid.cell(0, 0).set_content("0-0").set_row_span(1);
            grid.cell(1, 0).set_content("1-0");
            grid.cell(1, 1).set_content("1-1");

            let expected = concat!(
                "+-------+\n",
                "|  0-0  |\n",
                "+-------+\n",
                "|1-0|1-1|\n",
                "+---+---+\n"
            );

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_row_span_multilane() {
            let mut grid = Grid::new(4, 3);
            grid.cell(0, 0).set_content("first line").set_row_span(1);
            grid.cell(0, 2).set_content("e.g.");
            grid.cell(1, 0).set_content("0");
            grid.cell(1, 1).set_content("1");
            grid.cell(1, 2).set_content("2");
            grid.cell(2, 0).set_content("0");
            grid.cell(2, 1).set_content("1");
            grid.cell(2, 2).set_content("2");
            grid.cell(3, 0)
                .set_content("full last line")
                .set_row_span(2);

            let expected = concat!(
                "+------------+----+\n",
                "| first line |e.g.|\n",
                "+------------+----+\n",
                "|  0  |  1  |  2  |\n",
                "+-----+-----+-----+\n",
                "|  0  |  1  |  2  |\n",
                "+-----+-----+-----+\n",
                "| full last line  |\n",
                "+-----------------+\n"
            );

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_row_span_with_horizontal_ident() {
            let mut grid = Grid::new(3, 2);
            grid.cell(0, 0).set_content("0-0").set_row_span(1);
            grid.cell(1, 0).set_content("1-0").set_horizontal_ident(4);
            grid.cell(1, 1).set_content("1-1");
            grid.cell(2, 0).set_content("2-0");
            grid.cell(2, 1).set_content("2-1");

            let expected = concat!(
                "+---------------+\n",
                "|      0-0      |\n",
                "+---------------+\n",
                "|    1-0    |1-1|\n",
                "+-----------+---+\n",
                "|  2-0  |  2-1  |\n",
                "+-------+-------+\n",
            );

            assert_eq!(expected, grid.to_string());
        }

        #[test]
        fn render_row_span_with_odd_length() {
            let mut grid = Grid::new(2, 2);
            grid.cell(0, 0).set_content("3   ").set_row_span(1);
            grid.cell(1, 0).set_content("2");
            grid.cell(1, 1).set_content("3");

            let expected = concat!(
                "+-----+\n",
                "|3    |\n",
                "+-----+\n",
                "|2 |3 |\n",
                "+--+--+\n",
            );

            assert_eq!(expected, grid.to_string());
        }
    }

    #[test]
    // Might this behavior should be changed
    fn cell_formating_empty() {
        let mut cell = Cell::new();
        cell.set_content("").set_corner("-");

        let expected = concat!("--\n", "--");

        assert_eq!(expected, CellFormatter::new(&cell).boxed().format());
    }

    #[test]
    fn cell_formating_single() {
        let mut cell = Cell::new();
        cell.set_content("hello world").set_corner("-");

        let expected = concat!("-------------\n", "|hello world|\n", "-------------");

        assert_eq!(expected, CellFormatter::new(&cell).boxed().format());
    }

    #[test]
    fn cell_formating_multiline() {
        let mut cell = Cell::new();
        cell.set_content("hello\nworld").set_corner("-");

        let expected = concat!("-------\n", "|hello|\n", "|world|\n", "-------");

        assert_eq!(expected, CellFormatter::new(&cell).boxed().format());
    }

    #[test]
    fn cell_formating_multilane_forced() {
        let mut cell = Cell::new();
        cell.set_content("hello").set_corner("-");

        let expected = concat!("-------\n", "|hello|\n", "|     |\n", "-------");

        assert_eq!(
            expected,
            CellFormatter::new(&cell).boxed().height(2).format()
        );
    }

    #[test]
    fn empty_cell_formating_with_height_2() {
        let mut cell = Cell::new();
        cell.set_content("").set_corner("-");

        let expected = concat!("--\n", "||\n", "||\n", "--");
        let formated_cell = CellFormatter::new(&cell).boxed().height(2).format();

        assert_eq!(expected, formated_cell);
    }

    #[test]
    fn empty_cell_formating_with_height_1() {
        let mut cell = Cell::new();
        cell.set_content("").set_corner("-");

        let expected = concat!("--\n", "||\n", "--");
        let formated_cell = CellFormatter::new(&cell).boxed().height(1).format();

        assert_eq!(expected, formated_cell);
    }

    #[test]
    fn cell_formating_with_height_2() {
        let mut cell = Cell::new();
        cell.set_content("text").set_corner("-");

        let expected = concat!("------\n", "|text|\n", "|    |\n", "------");
        let formated_cell = CellFormatter::new(&cell).boxed().height(2).format();

        assert_eq!(expected, formated_cell);
    }

    #[test]
    fn cell_new_line_formating_with_height_2() {
        let mut cell = Cell::new();
        cell.set_content("\n").set_corner("-");

        let expected = concat!("--\n", "||\n", "||\n", "--");
        let formated_cell = CellFormatter::new(&cell).boxed().height(2).format();

        assert_eq!(expected, formated_cell);
    }
}
