use crossterm::event::KeyCode;

const LEFT_SHIFT: u8 = 0x12;

pub fn encode_tap(key: KeyCode) -> Option<Vec<u8>> {
    let (needs_shift, extended, code) = resolve(key)?;
    let mut bytes = Vec::with_capacity(8);

    if needs_shift {
        bytes.push(LEFT_SHIFT);
    }

    if extended {
        bytes.push(0xE0);
    }
    bytes.push(code);

    if extended {
        bytes.push(0xE0);
    }
    bytes.push(0xF0);
    bytes.push(code);
    if needs_shift {
        bytes.push(0xF0);
        bytes.push(LEFT_SHIFT);
    }

    Some(bytes)
}


fn resolve(key: KeyCode) -> Option<(bool, bool, u8)> {
    use KeyCode::*;
    match key {
        Char(c) => char_code(c),
        Enter => Some((false, false, 0x5A)),
        Backspace => Some((false, false, 0x66)),
        Tab => Some((false, false, 0x0D)),
        Esc => Some((false, false, 0x76)),

        Up => Some((false, true, 0x75)),
        Down => Some((false, true, 0x72)),
        Left => Some((false, true, 0x6B)),
        Right => Some((false, true, 0x74)),
        Insert => Some((false, true, 0x70)),
        Delete => Some((false, true, 0x71)),
        Home => Some((false, true, 0x6C)),
        End => Some((false, true, 0x69)),
        PageUp => Some((false, true, 0x7D)),
        PageDown => Some((false, true, 0x7A)),

        F(1) => Some((false, false, 0x05)),
        F(2) => Some((false, false, 0x06)),
        F(3) => Some((false, false, 0x04)),
        F(4) => Some((false, false, 0x0C)),
        F(5) => Some((false, false, 0x03)),
        F(6) => Some((false, false, 0x0B)),
        F(7) => Some((false, false, 0x83)),
        F(8) => Some((false, false, 0x0A)),
        F(9) => Some((false, false, 0x01)),
        F(10) => Some((false, false, 0x09)),
        F(11) => Some((false, false, 0x78)),
        F(12) => Some((false, false, 0x07)),

        _ => None,
    }
}

fn char_code(c: char) -> Option<(bool, bool, u8)> {
    let (shift, code) = match c {
        'a' => (false, 0x1C), 'A' => (true, 0x1C),
        'b' => (false, 0x32), 'B' => (true, 0x32),
        'c' => (false, 0x21), 'C' => (true, 0x21),
        'd' => (false, 0x23), 'D' => (true, 0x23),
        'e' => (false, 0x24), 'E' => (true, 0x24),
        'f' => (false, 0x2B), 'F' => (true, 0x2B),
        'g' => (false, 0x34), 'G' => (true, 0x34),
        'h' => (false, 0x33), 'H' => (true, 0x33),
        'i' => (false, 0x43), 'I' => (true, 0x43),
        'j' => (false, 0x3B), 'J' => (true, 0x3B),
        'k' => (false, 0x42), 'K' => (true, 0x42),
        'l' => (false, 0x4B), 'L' => (true, 0x4B),
        'm' => (false, 0x3A), 'M' => (true, 0x3A),
        'n' => (false, 0x31), 'N' => (true, 0x31),
        'o' => (false, 0x44), 'O' => (true, 0x44),
        'p' => (false, 0x4D), 'P' => (true, 0x4D),
        'q' => (false, 0x15), 'Q' => (true, 0x15),
        'r' => (false, 0x2D), 'R' => (true, 0x2D),
        's' => (false, 0x1B), 'S' => (true, 0x1B),
        't' => (false, 0x2C), 'T' => (true, 0x2C),
        'u' => (false, 0x3C), 'U' => (true, 0x3C),
        'v' => (false, 0x2A), 'V' => (true, 0x2A),
        'w' => (false, 0x1D), 'W' => (true, 0x1D),
        'x' => (false, 0x22), 'X' => (true, 0x22),
        'y' => (false, 0x35), 'Y' => (true, 0x35),
        'z' => (false, 0x1A), 'Z' => (true, 0x1A),

        '0' => (false, 0x45), ')' => (true, 0x45),
        '1' => (false, 0x16), '!' => (true, 0x16),
        '2' => (false, 0x1E), '@' => (true, 0x1E),
        '3' => (false, 0x26), '#' => (true, 0x26),
        '4' => (false, 0x25), '$' => (true, 0x25),
        '5' => (false, 0x2E), '%' => (true, 0x2E),
        '6' => (false, 0x36), '^' => (true, 0x36),
        '7' => (false, 0x3D), '&' => (true, 0x3D),
        '8' => (false, 0x3E), '*' => (true, 0x3E),
        '9' => (false, 0x46), '(' => (true, 0x46),

        ' ' => (false, 0x29),
        '-' => (false, 0x4E), '_' => (true, 0x4E),
        '=' => (false, 0x55), '+' => (true, 0x55),
        '[' => (false, 0x54), '{' => (true, 0x54),
        ']' => (false, 0x5B), '}' => (true, 0x5B),
        '\\' => (false, 0x5D), '|' => (true, 0x5D),
        ';' => (false, 0x4C), ':' => (true, 0x4C),
        '\'' => (false, 0x52), '"' => (true, 0x52),
        ',' => (false, 0x41), '<' => (true, 0x41),
        '.' => (false, 0x49), '>' => (true, 0x49),
        '/' => (false, 0x4A), '?' => (true, 0x4A),
        '`' => (false, 0x0E), '~' => (true, 0x0E),

        _ => return None,
    };
    Some((shift, false, code))
}
