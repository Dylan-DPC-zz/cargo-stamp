
use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Write, Seek, SeekFrom}};
use std::{error::Error, path::PathBuf, str::pattern::Pattern};
use std::fmt::{Display, Formatter, Error as FmtError};
use regex::Regex;

#[derive(Debug)]
pub struct Handler {
    path: PathBuf,
    file: File,
    contents: String
}

impl Handler {
    pub fn try_new<T: Into<PathBuf>>(path: T) -> Result<Handler, Box<dyn Error>>{
        let path = path.into();
        let file = OpenOptions::new().read(true).write(true).open(path.clone())?;
        Ok(Handler {
            path,
            file,
            contents: String::default()
        })
    }

    pub fn read(mut self) -> Result<Handler, Box<dyn Error>> {
        let mut buffer = BufReader::new(&self.file);
        let mut contents = String::new();
        buffer.read_to_string(&mut contents)?;
        self.contents = contents;

        Ok(self)
    }

    pub fn write(self, contents: &str) -> Result<(), Box<dyn Error>> {
        let mut file = &self.file;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write(contents.as_bytes())?;
        Ok(())
    }

    pub fn replace<P>(self, key: P, with: &str) -> Result<(), Box<dyn Error>>
    where P: for<'a> Pattern<'a>
    {
        let contents= &self.contents.replace(key, with);
        self.write(contents)
    }

    pub fn search_and_replace<'a>(self, key: &'a str, with: &str) -> Result<(), Box<dyn Error>> {
        let regex = Regex::new(format!(r"^.*?{}.*$", key).as_str()).unwrap();
        self.replace(&regex, with)
    }

    pub fn move_to<'a>(self, after: &'a str, key: &'a str) -> Result<(), Box<dyn Error>>{
        let mut contents: Vec<String> = self.contents.lines().map(|x| x.to_owned()).collect();

        let (index, _) = contents.find_regex(key)?;

        let (position, _cont) = contents.iter().enumerate().filter(|(_key, cont)| cont.contains(after)).last()
            .ok_or(CannotFindInFile { token: after.to_owned()})?;

        contents.move_elements(index, position, 1)?;
        let data = &contents.join("\n");
        self.write(data)
    }

    pub fn move_n_lines_to<'a>(self, after: &'a str, key: &'a str, n: usize, direction: Direction) -> Result<(), Box<dyn Error>> {
        let mut contents: Vec<String> = self.contents.lines().map(|x| x.to_owned()).collect();
        let (key_index, _) = contents.find_regex(key)?;
        let (index, source) = match direction {
            Direction::Above => (key_index - n, &contents[key_index-n ..= key_index]),
            Direction::Below => (key_index, &contents[key_index ..= key_index + n])
        };

        let (position, _cont) = contents.iter().enumerate().filter(|(_key, cont)| cont.contains(after)).last()
            .ok_or(CannotFindInFile { token: after.to_owned()})?;

        contents.move_elements(index, position, source.len())?;

        let data = &mut contents.join("\n");
        data.push_str("\n");
        self.write(data)
    }
}
trait VecExt {
    fn move_elements(&mut self, src: usize, dst: usize, n: usize) -> Result<(), Box<dyn Error>>;

    fn find_regex(&self, regex: &str) -> Result<(usize, String), Box<dyn Error>>;
}

impl VecExt for Vec<String>
{
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
        self.iter().enumerate().find(|(_, line)| {
            regex_key.find(*line).is_some()
        }).map(|(x,y)| (x, y.to_owned()))
            .ok_or(CannotFindInFile{ token: key.to_owned()}.into())
    }
}

#[derive(Debug)]
pub struct CannotFindInFile {
    pub token: String
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
        write!(f, "the number of places to move is larger than the length of the vector")
    }
}

impl Error for OutOfBounds {}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Above,
    Below
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, read_to_string};

    #[test]
    fn replace_replaces_in_a_file() {
        let mut dummy = File::create("dummy").expect("cannot open file for initial reading");
        dummy.write(b"foo bar baz ").expect("cannot write file");
        let file = Handler::try_new("dummy").unwrap().read().expect("cannot read file");
        file.replace("bar ", "").expect("cannot replace");
        let output: String = read_to_string("dummy").expect("cannot read file")
            .parse().expect("cannot parse");

        assert_eq!(output, "foo baz ");
    }

    #[test]
    fn replace_where_line_contains_a_key() {
        let mut dummy = File::create("dummy_contains").expect("cannot open file for initial reading");
        dummy.write(b"foo bar baz ").expect("cannot write file");
        let file = Handler::try_new("dummy_contains").unwrap().read().expect("cannot read file");
        file.search_and_replace("bar", "an entire new line");
        let output: String = read_to_string("dummy_contains").expect("cannot read file")
            .parse().expect("cannot parse");

        assert_eq!(output, "an entire new line");
    }

    #[test]
    fn move_to_moves_the_line_after_specified() {
        let mut dummy = File::create("dummy_move").expect("cannot open file for initial reading");
        dummy.write(b"foo\nbar baz\nqux\nquatre").expect("cannot write file");
        let file = Handler::try_new("dummy_move").unwrap().read().expect("cannot read file");
        file.move_to("qux", "bar").expect("cannot move file");
        let output: String = read_to_string("dummy_move").expect("cannot read file")
                                        .parse().expect("cannot parse");

        assert_eq!(output, "foo\nqux\nbar baz\nquatre");
    }

    #[test]
    fn move_n_lines_to_can_move_n_lines_before_the_key() {
        let mut dummy = File::create("dummy_move_many").expect("cannot open file for initial reading");
        dummy.write(b"foo\nbar\nbaz qux\nquux\n").expect("cannot write file");
        let file = Handler::try_new("dummy_move_many").unwrap().read().expect("cannot read file");
        file.move_n_lines_to("quux", "bar", 1, Direction::Above).expect("cannot move file");
        let output: String = read_to_string("dummy_move_many").expect("cannot read file")
            .parse().expect("cannot parse");

        assert_eq!(output, "baz qux\nquux\nfoo\nbar\n");
    }
}

