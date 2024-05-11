use chrono::{Datelike, NaiveDateTime};
use exif::{In, Tag};
use globwalk::{GlobError, GlobWalker};
use std::collections::BTreeMap;
use std::error::Error;
use std::io;
use std::path::PathBuf;

pub mod arguments;
use crate::arguments::Arguments;

const PATTERNS: [&str; 5] = ["*.png", "*.jpg", "*.jpeg", "*.heic", ".mov"];

#[derive(Debug, PartialEq)]
struct Image {
    path: PathBuf,
}

impl Image {
    fn new(path: PathBuf) -> Self {
        Image { path }
    }
}

enum Tree {
    YearMonth(BTreeMap<(i32, u32), Vec<Image>>),
    Year(BTreeMap<i32, Vec<Image>>),
    Month(BTreeMap<u32, Vec<Image>>),
}

impl Tree {
    fn insert(&mut self, datetime: (i32, u32), image: Image) {
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

    fn size(&self) -> usize {
        match self {
            Tree::YearMonth(tree) => tree.values().map(Vec::len).sum(),
            Tree::Year(tree) => tree.values().map(Vec::len).sum(),
            Tree::Month(tree) => tree.values().map(Vec::len).sum(),
        }
    }
}

fn build_tree(years: &bool, months: &bool) -> Tree {
    // Args validated, one of these three types will always appear
    if *months && *years {
        Tree::YearMonth(BTreeMap::new())
    } else if *years {
        Tree::Year(BTreeMap::new())
    } else {
        Tree::Month(BTreeMap::new())
    }
}

fn build_glob_walker(path: &PathBuf, patterns: &[&str]) -> Result<GlobWalker, GlobError> {
    globwalk::GlobWalkerBuilder::from_patterns(path, patterns)
        .max_depth(4)
        .follow_links(true)
        .case_insensitive(true)
        .build()
}

fn find(walker: GlobWalker, tree: &mut Tree) -> Result<(), Box<dyn Error>> {
    // Convert to peekable itertor to check if empty
    let mut images = walker.into_iter().filter_map(Result::ok).peekable();

    if images.peek().is_none() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Did not find any media with metadata.",
        )));
    }

    for image in images {
        let path = image.path().to_path_buf();

        if let Some(datetime) = get_datetime_original(&path) {
            tree.insert(datetime, Image::new(path));
        } else {
            // Insert pics without metadata under (0, 0)
            tree.insert((0, 0), Image::new(path));
        }
    }

    Ok(())
}

fn get_datetime_original(path: &PathBuf) -> Option<(i32, u32)> {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);

    let exifreader = exif::Reader::new();
    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => exif,
        Err(_) => return None,
    };

    match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        None => None,
        Some(field) => {
            let datetime_str = field.display_value().with_unit(&exif).to_string();
            NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| (dt.year(), dt.month()))
        }
    }
}

// Means that function will return a type that implements the Error trait
pub fn run(args: &Arguments) -> Result<(), Box<dyn Error>> {
    let walker = build_glob_walker(&args.path, &PATTERNS)?;
    let mut tree = build_tree(&args.years, &args.months);

    find(walker, &mut tree)?;

    println!("Found {} pieces of media with metadata", tree.size());

    match tree {
        Tree::YearMonth(t) => {
            for ((year, month), images) in t {
                println!("Year: {}, Month: {}", year, month);
                for image in images {
                    println!("  Image: {:?}", image.path);
                }
            }
        }
        Tree::Year(t) => {
            for (year, images) in t {
                println!("Year: {}", year);
                for image in images {
                    println!("  Image: {:?}", image.path);
                }
            }
        }
        Tree::Month(t) => {
            for (month, images) in t {
                println!("Month: {}", month);
                for image in images {
                    println!("  Image: {:?}", image.path);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use exif::experimental;
    use exif::{Field, In, Tag, Value};
    use image::RgbImage;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufWriter;
    use tempfile::TempDir;

    fn create_image_with_metadata(path: &PathBuf, datetime: &str) -> Result<(), Box<dyn Error>> {
        // Create and save image
        let img = RgbImage::new(32, 32);
        img.save(path)?;

        // Open image to add EXIF data
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Add the DateTimeOriginal tag
        let mut exif_writer = experimental::Writer::new();
        let datetime_original = Field {
            tag: Tag::DateTimeOriginal,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![datetime.as_bytes().to_vec()]),
        };

        exif_writer.push_field(&datetime_original);
        exif_writer.write(&mut writer, false)?;

        Ok(())
    }

    fn touch(
        dir: &TempDir,
        names: impl IntoIterator<Item = impl AsRef<str>>,
        datetime: Option<&str>,
    ) {
        for name in names {
            let file_path = dir.path().join(name.as_ref());

            if let Some(datetime) = datetime {
                create_image_with_metadata(&file_path, datetime)
                    .expect("Failed to create a test file with metadata");
            } else {
                File::create(&file_path).expect("Failed to create a test file");
            }
        }
    }

    #[test]
    fn not_dir() {
        // Ensure args has error on invalid directory
        let dir = TempDir::new().expect("Failed to create temporary folder");
        let dir_path = dir.path();

        touch(&dir, ["f.txt"], None);

        let path = dir_path.join("f.txt");

        let args = Arguments {
            path,
            months: true,
            years: true,
        };

        let args = Arguments::validate(&args);

        assert!(
            args.is_err(),
            "Expected an error for a file path used as a directory"
        );
    }

    #[test]
    fn invalid_path() {
        // Ensure args has error on invalid path
        let path = PathBuf::from("bleh");

        let args = Arguments {
            path,
            months: true,
            years: true,
        };

        let args = Arguments::validate(&args);

        assert!(
            args.is_err(),
            "Expected an error for a non-existent file path used as a directory"
        );
    }
    #[test]
    fn invalid_sort_flags() {
        // Ensure args has error on invalid path
        let path = PathBuf::from("bleh");

        let args = Arguments {
            path,
            months: false,
            years: false,
        };

        let args = Arguments::validate(&args);

        assert!(
            args.is_err(),
            "Expected an error for neither sort option selected"
        );
    }

    #[test]
    fn globwalker_invalid_patterns() {
        let dir = TempDir::new().expect("Failed to create temporary folder");
        let dir_path = PathBuf::from(dir.path());
        let invalid_patterns = ["\\", ""];

        let walker = build_glob_walker(&dir_path, &invalid_patterns);

        assert!(
            walker.is_err(),
            "Expected an error for invalid search patterns"
        );
    }
    #[test]
    fn globwalker_valid_patterns() {
        let dir = TempDir::new().expect("Failed to create temporary folder");
        let dir_path = PathBuf::from(dir.path());

        let walker = build_glob_walker(&dir_path, &PATTERNS);

        assert!(walker.is_ok(), "Expected OK for valid search patterns");
    }
    #[test]
    fn build_year_month_tree() {
        let years = true;
        let months = true;

        let tree = build_tree(&years, &months);

        match tree {
            Tree::YearMonth(_) => println!("Tree is an instance of YearMonth"),
            _ => panic!("Expected Tree to be YearMonth variant"),
        }
    }
    #[test]
    fn build_year_tree() {
        let years = true;
        let months = false;

        let tree = build_tree(&years, &months);

        match tree {
            Tree::Year(_) => println!("Tree is an instance of Year"),
            _ => panic!("Expected Tree to be Year variant"),
        }
    }
    #[test]
    fn build_month_tree() {
        let years = false;
        let months = true;

        let tree = build_tree(&years, &months);

        match tree {
            Tree::Month(_) => println!("Tree is an instance of Month"),
            _ => panic!("Expected Tree to be Month variant"),
        }
    }

    // #[test]
    // fn find_no_existing_media() {
    //     // Ensure error occurs when no media found
    //     let dir = TempDir::new().expect("Failed to create temporary folder");
    //     let dir_path = dir.path().to_path_buf();

    //     let walker = match build_glob_walker(&dir_path) {
    //         Ok(walker) => walker,
    //         Err(msg) => panic!("Error building walker: {}", msg),
    //     };

    //     let results = find(walker);

    //     assert!(
    //         results.is_err(),
    //         "Expected an error for a directory without any media"
    //     );
    // }

    // #[test]
    // fn find_existing_media() {
    //     // Ensure media is found (Metadata not examined)
    //     let dir = TempDir::new().expect("Failed to create temporary folder");
    //     let dir_path = dir.path().to_path_buf();
    //     let files = ["a.png", "b.PNG", "c.jpg", "d.JPG", "e.jpeg", "f.JPEG"];

    //     // Need metadata or else find will error
    //     touch(&dir, files, Some("2022:01:01 00:00:00"));

    //     let walker = match build_glob_walker(&dir_path) {
    //         Ok(walker) => walker,
    //         Err(msg) => panic!("Error building walker: {}", msg),
    //     };

    //     let results = match find(walker) {
    //         Ok(results) => results,
    //         Err(msg) => panic!("Error getting results: {}", msg),
    //     };

    //     let result_set: HashSet<PathBuf> = results.into_iter().map(|img| img.path).collect();
    //     let expected_set: HashSet<PathBuf> = files.into_iter().map(|f| dir_path.join(f)).collect();

    //     assert_eq!(result_set, expected_set, "Expected OK results");
    // }

    #[test]
    fn find_existing_datetime() {
        // Ensure that datetimes found are as expected
        let dir = TempDir::new().expect("Failed to create temporary folder");
        let dir_path = dir.path().to_path_buf();
        let files = ["a.png", "b.PNG", "c.jpg", "d.JPG", "e.jpeg", "f.JPEG"];

        // Need metadata or else find will error
        touch(&dir, files, Some("2024:01:01 00:00:00"));

        // Collect datetimes
        let datetimes: HashSet<Option<(i32, u32)>> = files
            .iter()
            .map(|name| dir_path.join(name))
            .map(|f| get_datetime_original(&f))
            .collect();

        let expected_datetimes: HashSet<Option<(i32, u32)>> = HashSet::from([Some((2024, 1))]);
        assert_eq!(datetimes, expected_datetimes, "Expected datetime results");
    }

    #[test]
    fn find_non_existing_datetime() {
        // Ensure that no datetime is correctly handled
        let dir = TempDir::new().expect("Failed to create temporary folder");
        let dir_path = dir.path().to_path_buf();
        let files = ["a.png", "b.PNG", "c.jpg", "d.JPG", "e.jpeg", "f.JPEG"];

        // Create files without metadata
        touch(&dir, files, None);

        // Collect datetimes
        let datetimes: HashSet<Option<(i32, u32)>> = files
            .iter()
            .map(|name| dir_path.join(name))
            .map(|f| get_datetime_original(&f))
            .collect();

        let expected_datetimes: HashSet<Option<(i32, u32)>> = HashSet::from([None]);
        assert_eq!(datetimes, expected_datetimes, "Expected datetime results");
    }
}
