use std::iter::Peekable;

const UNRECOGNIZED: &str = "unrecognized input. try 'help' or 'quit'";

#[derive(Debug)]
pub enum Source {
    Waste,
    Tableau { column: usize, row: usize },
}

#[derive(Debug)]
pub enum Destination {
    Foundation(usize),
    Tableau(usize),
}

#[derive(Debug)]
pub enum Action {
    Quit,
    Help,
    Draw,
    Move(Source, Destination),
    QuickMove(Source),
}

impl std::str::FromStr for Action {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Action, Self::Err> {
        parse_action(s)
    }
}

fn parse_source(chars: &mut Peekable<impl Iterator<Item=char>>)
    -> Result<Source, &'static str>
{
    let c = match chars.next() {
        Some(c) => c,
        None => return Err(UNRECOGNIZED),
    };
    match c {
        'W' => return Ok(Source::Waste),
        '0' => return Err("can't move from the foundation"),
        '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            let column = (c as u32 - '1' as u32) as usize;
            if chars.peek().is_none() {
                return Err("missing a tableau row letter");
            }
            if let Some(row) = get_int(chars, 'A', 'Z') {
                return Ok(Source::Tableau { column, row });
            }
        }
        _ => (),
    }
    Err(UNRECOGNIZED)
}

fn parse_destination(chars: &mut Peekable<impl Iterator<Item=char>>)
    -> Result<Destination, &'static str>
{
    let c = match chars.next() {
        Some(c) => c,
        None => return Err(UNRECOGNIZED),
    };
    match c {
        '0' => if let Some(idx) = get_int(chars, 'A', 'D') {
            return Ok(Destination::Foundation(idx));
        }
        '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            let column = (c as u32 - '1' as u32) as usize;
            if chars.peek().is_some() {
                return Err("extra input after tableau column number");
            }
            return Ok(Destination::Tableau(column));
        }
        'W' => return Err("can't move to the waste"),
        _ => (),
    }
    Err(UNRECOGNIZED)
}

fn get_int(mut chars: impl Iterator<Item = char>, min: char, max: char) -> Option<usize> {
    if let Some(c) = chars.next() {
        if c as u32 >= min as u32 && c as u32 <= (max as u32) {
            return Some((c as u32 - min as u32) as usize);
        }
    }
    None
}

fn parse_action(s: &str) -> Result<Action, &'static str> {
    match s.to_ascii_uppercase().as_str() {
        "" => return Err("enter 'quit' to exit, or try 'help'"),
        "Q" | "QUIT" => return Ok(Action::Quit),
        "HELP" => return Ok(Action::Help),
        "DD" => return Ok(Action::Draw),
        _ => (),
    }

    let mut chars = s.chars().map(|c| c.to_ascii_uppercase()).peekable();
    let source = parse_source(&mut chars)?;

    if chars.peek().is_none() {
        // let the game figure out if this is valid or not
        return Ok(Action::QuickMove(source));
    }

    let dest = parse_destination(&mut chars)?;

    if chars.peek().is_some() {
        return Err("unrecognized extra input after move");
    }

    Ok(Action::Move(source, dest))
}
