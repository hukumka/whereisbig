mod crawler;

use std::fmt;
use std::io::{self, Write};
use std::{io::StdoutLock, path::PathBuf};

use crate::crawler::{Crawler, DirTree};
use clap::Parser;
use reformation::Reformation;

#[derive(Debug, PartialEq, Clone, Reformation)]
#[reformation("{value}{unit}", fromstr = true)]
struct Size {
    #[reformation(r"\d+(\.\d+)?")]
    value: f64,
    unit: SizeUnit,
}

#[derive(Debug, PartialEq, Clone, Reformation)]
#[reformation(fromstr = true)]
enum SizeUnit {
    #[reformation(r"b?")]
    Byte,
    #[reformation(r"K")]
    Kilobyte,
    #[reformation(r"M")]
    Megabyte,
    #[reformation(r"G")]
    Gigabyte,
}

impl Size {
    fn size_in_bytes(&self) -> u64 {
        let size = Self::unit_multiplier(&self.unit) * self.value;
        return size as u64;
    }

    fn unit_multiplier(unit: &SizeUnit) -> f64 {
        match unit {
            SizeUnit::Byte => 1.0,
            SizeUnit::Kilobyte => 1e3,
            SizeUnit::Megabyte => 1e6,
            SizeUnit::Gigabyte => 1e9,
        }
    }

    fn from_size_in_bytes(size: u64, unit: SizeUnit) -> Self {
        Self {
            value: (size as f64) / Self::unit_multiplier(&unit),
            unit,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

impl fmt::Display for SizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit_str = match &self {
            SizeUnit::Byte => "b",
            SizeUnit::Kilobyte => "K",
            SizeUnit::Megabyte => "M",
            SizeUnit::Gigabyte => "G",
        };
        write!(f, "{}", unit_str)
    }
}

#[derive(Parser, Debug)]
struct Args {
    path: PathBuf,
    #[clap(short = 's', long)]
    dir_size: Size,
    #[clap(short = 'u', long, default_value = "K")]
    diplay_unit: SizeUnit,
}

struct TreeRenderer<'a> {
    lock: StdoutLock<'a>,
    unit: &'a SizeUnit,
}

impl<'a> TreeRenderer<'a> {
    fn new(unit: &'a SizeUnit, lock: StdoutLock<'a>) -> Self {
        Self { unit, lock }
    }

    fn render_tree(&mut self, forest: &[DirTree]) -> io::Result<()> {
        for tree in forest {
            self.render_tree_recursion(0, tree)?
        }
        Ok(())
    }

    fn render_tree_recursion(&mut self, level: u32, tree: &DirTree) -> io::Result<()> {
        self.render_record(level, tree)?;
        for child in &tree.children {
            self.render_tree_recursion(level + 1, child)?;
        }
        Ok(())
    }

    fn render_record(&mut self, level: u32, record: &DirTree) -> io::Result<()> {
        for _ in 0..level {
            write!(&mut self.lock, "\t")?;
        }
        writeln!(
            &mut self.lock,
            "{} {}",
            record.path.file_name().unwrap().to_str().unwrap(),
            Size::from_size_in_bytes(record.size, self.unit.clone()),
        )
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let tree = Crawler::new(args.dir_size.size_in_bytes()).walk(&args.path)?;
    TreeRenderer::new(&args.diplay_unit, io::stdout().lock()).render_tree(&tree)?;
    Ok(())
}
