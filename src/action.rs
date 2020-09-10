#[derive(Debug)]
pub enum Location {
    Waste,
    Foundation(usize),
    Tableau { column: usize, row: Option<usize> },
}

#[derive(Debug)]
pub enum Action {
    Quit,
    Draw,
    Move(Location, Location),
    QuickMove(Location),
}

impl std::str::FromStr for Action {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Action, Self::Err> {
        parse_action(s)
    }
}

fn parse_location(mut chars: impl Iterator<Item=char>)
    -> Result<Location, &'static str>
{
    let c = chars.next().unwrap();
    match c {
        'W' => return Ok(Location::Waste),
        '0' => if let Some(idx) = get_int(chars, 'A', 'D') {
            return Ok(Location::Foundation(idx));
        }
        '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            let column = (c as u32 - '1' as u32) as usize;
            let next = chars.next();
            if next.is_none() {
                // No row specified: refers to the column as a whole. Only valid when used as a
                // destination.
                return Ok(Location::Tableau { column, row: None });
            }
            if let Some(row) = get_int(next.iter().cloned(), 'A', 'Z') {
                return Ok(Location::Tableau { column, row: Some(row) });
            }
        }
        _ => (),
    }
    Err("unrecognized input")
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
        "" => return Err("enter 'quit' to exit"),
        "Q" | "QUIT" => return Ok(Action::Quit),
        "DD" => return Ok(Action::Draw),
        _ => (),
    }

    let mut chars = s.chars().map(|c| c.to_ascii_uppercase()).peekable();
    let source = parse_location(&mut chars)?;
    if let Location::Tableau { column: _, row } = source {
        if row.is_none() {
            return Err("unrecognized input");
        }
    }

    if chars.peek().is_none() {
        // let the game figure out if this is valid or not
        return Ok(Action::QuickMove(source));
    }

    let dest = parse_location(&mut chars)?;
    if let Location::Waste = dest {
        return Err("can't move a card to the waste pile");
    }

    Ok(Action::Move(source, dest))
}
