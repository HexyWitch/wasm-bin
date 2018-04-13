use std::io;

#[derive(Debug)]
pub enum Error {
    ReadLineError(io::Error),
}

pub fn prompt_confirm(text: &str) -> Result<bool, Error> {
    println!("{}", text);

    let read_in = || {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .map_err(Error::ReadLineError)?;
        Ok(match buf.trim_right() {
            "y" | "Y" => Some(true),
            "n" | "N" => Some(false),
            _ => None,
        })
    };
    loop {
        match read_in()? {
            Some(v) => return Ok(v),
            _ => {}
        }
    }
}
