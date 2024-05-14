use crate::image::Image;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(PartialEq,Debug)]
pub enum Tree {
    YearMonth(BTreeMap<(i32, u32), Vec<Image>>),
    Year(BTreeMap<i32, Vec<Image>>),
    Month(BTreeMap<u32, Vec<Image>>),
}

impl Tree {
    pub fn insert(&mut self, datetime: (i32, u32), image: Image) {
        match self {
            Tree::YearMonth(tree) => {
                tree.entry(datetime).or_insert_with(Vec::new).push(image);
            }
            Tree::Year(tree) => {
                let (year, _) = datetime;
                tree.entry(year).or_insert_with(Vec::new).push(image);
            }
            Tree::Month(tree) => {
                let (_, month) = datetime;
                tree.entry(month).or_insert_with(Vec::new).push(image);
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Tree::YearMonth(tree) => tree.values().map(Vec::len).sum(),
            Tree::Year(tree) => tree.values().map(Vec::len).sum(),
            Tree::Month(tree) => tree.values().map(Vec::len).sum(),
        }
    }

    pub fn print(&self) {
        match self {
            Tree::YearMonth(tree) => {
                for ((year, month), images) in tree {
                    println!("Year: {}, Month: {}", year, month);
                    for image in images {
                        println!("  Image: {:?}", image.path);
                    }
                }
            }
            Tree::Year(tree) => {
                for (year, images) in tree {
                    println!("Year: {}", year);
                    for image in images {
                        println!("  Image: {:?}", image.path);
                    }
                }
            }
            Tree::Month(tree) => {
                for (month, images) in tree {
                    println!("Month: {}", month);
                    for image in images {
                        println!("  Image: {:?}", image.path);
                    }
                }
            }
        }
    }

    pub fn save(&self, dest: &PathBuf) -> io::Result<()> {
        match self {
            Tree::YearMonth(tree) => {
                for ((year, month), images) in tree {
                    let dir = dest.join(year.to_string()).join(get_month(month));
                    fs::create_dir_all(&dir)?;

                    for image in images {
                        let dest = dir.join(&image.name);
                        fs::copy(&image.path, &dest)?;
                    }
                }
            }
            Tree::Year(tree) => {
                for (year, images) in tree {
                    let dir = dest.join(year.to_string());
                    fs::create_dir_all(&dir)?;

                    for image in images {
                        let dest = dir.join(&image.name);
                        fs::copy(&image.path, &dest)?;
                    }
                }
            }
            Tree::Month(tree) => {
                for (month, images) in tree {
                    let dir = dest.join(get_month(month));
                    println!("dir: {:?}", &dir);
                    fs::create_dir_all(&dir)?;

                    for image in images {
                        let dest = dir.join(&image.name);
                        fs::copy(&image.path, &dest)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_month(month: &u32) -> String {
    match month {
        1 => String::from("January"),
        2 => String::from("February"),
        3 => String::from("March"),
        4 => String::from("April"),
        5 => String::from("May"),
        6 => String::from("June"),
        7 => String::from("July"),
        8 => String::from("August"),
        9 => String::from("September"),
        10 => String::from("October"),
        11 => String::from("November"),
        12 => String::from("December"),
        _ => String::from("Unknown"),
    }
}

pub fn build_tree(years: &bool, months: &bool) -> Tree {
    match (years, months) {
        (true, true) => Tree::YearMonth(BTreeMap::new()),
        (true, false) => Tree::Year(BTreeMap::new()),
        (false, true) => Tree::Month(BTreeMap::new()),
        _ => unreachable!("Invalid combination of years and months"),
    }
}
