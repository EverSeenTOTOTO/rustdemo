struct Mistake {
    path: &'static str,

    text: String,
    locations: Vec<usize>,
}

impl Mistake {
    fn line_bounds(&self, index: usize) -> (usize, usize) {
        let len = self.text.len();

        let before = &self.text[..index];
        let start = before.rfind("\n").map(|x| x + 1).unwrap_or(0);

        let after = &self.text[index + 1..];
        let end = after.find("\n").map(|x| x + index + 1).unwrap_or(len);

        (start, end)
    }
}

impl std::fmt::Display for Mistake {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for &location in &self.locations {
            let (start, end) = self.line_bounds(location);
            let line = &self.text[start..end];

            let line_number = self.text[..start].matches("\n").count() + 1;
            let comma_index = location - start;

            write!(f, "{}: commas are forbidden:\n\n", self.path)?;

            // print the line, with line number
            write!(f, "{:>8} | {}\n", line_number, line)?;

            // indicate where the comma is
            write!(f, "{}^\n\n", " ".repeat(11 + comma_index))?;
        }
        Ok(())
    }
}

fn check_file(path: &'static str) -> Result<Option<Mistake>, std::io::Error> {
    let text = std::fs::read_to_string(path)?;

    let locations: Vec<_> = text.match_indices(",").map(|(index, _)| index).collect();

    Ok(if locations.is_empty() {
        None
    } else {
        Some(Mistake {
            path,
            text,
            locations,
        })
    })
}

pub fn test_read_file() -> Result<(), std::io::Error> {
    let paths = ["Cargo.lock"];

    // check all documents
    let mut results = vec![];
    for path in &paths {
        let result = check_file(path)?;
        results.push(result);
    }

    // report them all
    for result in results {
        if let Some(mistake) = result {
            println!("{}", mistake);
        }
    }

    Ok(())
}

pub fn test_read_stdin() -> Result<(), std::io::Error> {
    let mut buffer = String::with_capacity(1024);
    std::io::stdin().read_line(&mut buffer)?;

    let locations: Vec<_> = buffer.match_indices(",").map(|(index, _)| index).collect();

    if locations.is_empty() {
        println!("No comma found");
    } else {
        println!("{}", Mistake {
            path: "stdin",
            text: buffer,
            locations,
        });
    }
    Ok(())
}

pub fn test_read_dev_zero() -> Result<(), std::io::Error> {
    use std::io::{Write,Read};

    let mut file = std::fs::File::open("makefile")?;
    let mut zero = std::fs::File::open("/dev/zero")?;
    let mut str = String::with_capacity(1024);
    let mut buffer = [0; 10];

    file.read(&mut buffer)?;
    println!("{:?}", buffer);

    zero.read(&mut buffer)?;
    println!("{:?}", buffer);

    Ok(())
}
