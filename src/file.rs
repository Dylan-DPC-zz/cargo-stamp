use regex::Regex;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::{error::Error, path::PathBuf, str::pattern::Pattern};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write, Lines},
};

#[derive(Debug)]
pub struct Handler {
    path: PathBuf,
    file: File,
    contents: String,
}

impl Handler {
    pub fn try_new<T: Into<PathBuf>>(path: T) -> Result<Handler, Box<dyn Error>> {
        let path = path.into();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.clone())?;
        Ok(Handler {
            path,
            file,
            contents: String::default(),
        })
    }

    pub fn read(mut self) -> Result<Handler, Box<dyn Error>> {
        let mut buffer = BufReader::new(&self.file);
        let mut contents = String::new();
        buffer.read_to_string(&mut contents)?;
        self.contents = contents;

        Ok(self)
    }

    pub fn write(&mut self, contents: &str) -> Result<(), Box<dyn Error>> {
        let mut file = &self.file;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write(contents.as_bytes())?;
        self.contents = contents.to_owned();
        Ok(())
    }

    pub fn replace_all_occurrences<P>(&mut self, key: P, to: &str) -> Result<(), Box<dyn Error>>
    where
        P: for<'a> Pattern<'a>,
    {
        let contents = &self.contents.replace(key, to);
        self.write(contents)
    }

    pub fn search_and_replace(&mut self, key: &str, to: &str) -> Result<(), Box<dyn Error>> {
        let regex = Regex::new(format!(r"^.*?{}.*$", key).as_str()).unwrap();
        self.replace_all_occurrences(&regex, to)
    }

    fn replace<F: Fn((usize, &str)) -> String>(
        &mut self,
        at: usize,
        map: F,
    ) -> Result<(), Box<dyn Error>> {
        let contents = self
            .contents
            .lines()
            .enumerate()
            .map(map)
            .collect::<String>();

        self.write(&contents)
    }

    pub fn replace_at(&mut self, at: usize, from: &str, to: &str) -> Result<(), Box<dyn Error>> {
        self.replace(at, |(i, line)| {
            let mut line = if i == at {
                line.replace(from, to)
            } else {
                line.to_owned()
            };
            line.push_str("\n");
            line
        })
    }

    pub fn replace_line(&mut self, at: usize, to: &str) -> Result<(), Box<dyn Error>> {
        self.replace(at, |(i, line)| {
            let mut line = if i == at {
                line.replace(line, to)
            } else {
                line.to_owned()
            };

            line.push_str("\n");
            line
        })
    }

    pub fn move_to<'a>(&mut self, after: &'a str, key: &'a str) -> Result<usize, Box<dyn Error>> {
        let mut contents: Vec<String> = self.contents.lines().map(|x| x.to_owned()).collect();

        let (index, _) = contents.find_regex(key)?;

        let (position, _cont) = contents
            .iter()
            .enumerate()
            .filter(|(_key, cont)| cont.contains(after))
            .last()
            .ok_or(CannotFindInFile {
                token: after.to_owned(),
            })?;

        contents.move_elements(index, position, 1)?;
        let data = &contents.join("\n");
        self.write(data)?;

        Ok(index)
    }

    pub fn lines(&self) -> Vec<String> {
        self.contents.lines().map(|x| x.to_owned()).collect()
    }

    pub fn move_n_lines_to<'a>(
        &mut self,
        after: &'a str,
        key: &'a str,
        n: usize,
        direction: Direction,
    ) -> Result<usize, Box<dyn Error>> {
        let mut contents = self.lines();
        let (key_index, _) = contents.find_regex(key)?;

        let (start, end) = match direction {
            Direction::Above => (key_index - n, key_index),
            Direction::Below => (key_index, key_index + n),
        };

        if &contents[start - 1] == "" && &contents[end + 1] == "" {
            contents.remove(end + 1);
        }

        let position = position(&contents, after)?;
        let mut source = &mut contents[start..=end];
        let len = source.len();

        contents.move_elements(start, position, len)?;

        let data = &mut contents.join("\n");
        data.push_str("\n");
        self.write(data)?;

        Ok(position)
    }

    pub fn nth(&self, n: usize) -> Option<&str> {
        self.contents.lines().nth(n)
    }
}
trait VecExt {
    fn move_elements(&mut self, src: usize, dst: usize, n: usize) -> Result<(), Box<dyn Error>>;

    fn find_regex(&self, regex: &str) -> Result<(usize, String), Box<dyn Error>>;
}

impl VecExt for Vec<String> {
    fn move_elements(&mut self, src: usize, dst: usize, n: usize) -> Result<(), Box<dyn Error>> {
        if n > self.len() {
            Err(OutOfBounds {}.into())
        } else {
            if dst < src {
                self[dst..=src].rotate_right(n);
            } else {
                self[src..=dst].rotate_left(n);
            }

            Ok(())
        }
    }



    fn find_regex(&self, key: &str) -> Result<(usize, String), Box<dyn Error>> {
        let regex_key = Regex::new(format!(r"^.*?{}.*$", key).as_str())?;
        self.iter()
            .enumerate()
            .find(|(_, line)| regex_key.find(*line).is_some())
            .map(|(x, y)| (x, y.to_owned()))
            .ok_or(
                CannotFindInFile {
                    token: key.to_owned(),
                }
                .into(),
            )
    }
}

pub fn position(contents: &[String], after: &str) -> Result<usize, Box<dyn Error>> {
    Ok(contents
        .iter()
        .enumerate()
        .filter(|(_key, cont)| cont.contains(after))
        .last()
        .ok_or(CannotFindInFile {
            token: after.to_owned(),
        })?.0)
}
#[derive(Debug)]
pub struct CannotFindInFile {
    pub token: String,
}

impl Display for CannotFindInFile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "cannot find the token {} in the file ", self.token)
    }
}

impl Error for CannotFindInFile {}

#[derive(Debug)]
pub struct OutOfBounds {}

impl Display for OutOfBounds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(
            f,
            "the number of places to move is larger than the length of the vector"
        )
    }
}

impl Error for OutOfBounds {}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Above,
    Below,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_to_string, File};

    #[test]
    fn replace_replaces_in_a_file() {
        let mut dummy = File::create("dummy").expect("cannot open file for initial reading");
        dummy.write(b"foo bar baz ").expect("cannot write file");
        let mut file = Handler::try_new("dummy")
            .unwrap()
            .read()
            .expect("cannot read file");
        file.replace_all_occurrences("bar ", "")
            .expect("cannot replace");
        let output: String = read_to_string("dummy")
            .expect("cannot read file")
            .parse()
            .expect("cannot parse");

        assert_eq!(output, "foo baz ");
    }

    #[test]
    fn replace_where_line_contains_a_key() {
        let mut dummy =
            File::create("dummy_contains").expect("cannot open file for initial reading");
        dummy.write(b"foo bar baz ").expect("cannot write file");
        let mut file = Handler::try_new("dummy_contains")
            .unwrap()
            .read()
            .expect("cannot read file");
        file.search_and_replace("bar", "an entire new line");
        let output: String = read_to_string("dummy_contains")
            .expect("cannot read file")
            .parse()
            .expect("cannot parse");

        assert_eq!(output, "an entire new line");
    }

    #[test]
    fn move_to_moves_the_line_after_specified() {
        let mut dummy = File::create("dummy_move").expect("cannot open file for initial reading");
        dummy
            .write(b"foo\nbar baz\nqux\nquatre")
            .expect("cannot write file");
        let mut file = Handler::try_new("dummy_move")
            .unwrap()
            .read()
            .expect("cannot read file");
        file.move_to("qux", "bar").expect("cannot move file");
        let output: String = read_to_string("dummy_move")
            .expect("cannot read file")
            .parse()
            .expect("cannot parse");

        assert_eq!(output, "foo\nqux\nbar baz\nquatre");
    }

    #[test]
    fn move_n_lines_to_can_move_n_lines_before_the_key() {
        let mut dummy =
            File::create("dummy_move_many").expect("cannot open file for initial reading");
        dummy
            .write(b"foo\nbar\nbaz qux\nquux\n")
            .expect("cannot write file");
        let mut file = Handler::try_new("dummy_move_many")
            .unwrap()
            .read()
            .expect("cannot read file");
        file.move_n_lines_to("quux", "bar", 1, Direction::Above)
            .expect("cannot move file");
        let output: String = read_to_string("dummy_move_many")
            .expect("cannot read file")
            .parse()
            .expect("cannot parse");

        assert_eq!(output, "baz qux\nquux\nfoo\nbar\n");
    }

    #[test]
    fn replace_at_replaces_at_a_position() {
        let mut dummy =
            File::create("dummy_replace_at").expect("cannot open file for initial reading");
        dummy
            .write(b"foo\nbar\nbaz qux\nquux\n")
            .expect("cannot write file");
        let mut file = Handler::try_new("dummy_replace_at")
            .unwrap()
            .read()
            .expect("cannot read file");
        file.replace_at(2, "baz", "changed");
        let output: String = read_to_string("dummy_replace_at")
            .expect("cannot read file")
            .parse()
            .expect("cannot parse");

        assert_eq!(output, "foo\nbar\nchanged qux\nquux\n");
    }
}
