/// Edit instruction sent to the display server / input target on Linux.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditCommand {
    pub backspace_count: usize,
    pub chars: Vec<u32>,
}

impl EditCommand {
    /// Resets the EditCommand for reuse.
    pub fn clear(&mut self) {
        self.backspace_count = 0;
        self.chars.clear();
    }

    /// Converts the Unicode char data into a UTF-8 String payload.
    pub fn to_string(&self) -> String {
        let mut s = String::with_capacity(self.chars.len() * 4);
        for &code in &self.chars {
            if let Some(ch) = char::from_u32(code) {
                s.push(ch);
            }
        }
        s
    }
}

/// Generates edit payload commands given backspace count and UTF-32 raw characters.
pub fn generate_edit_command(backspace_count: usize, raw_chars: &[u32], char_count: usize) -> EditCommand {
    let mut cmd = EditCommand {
        backspace_count,
        chars: Vec::with_capacity(char_count),
    };
    for i in (0..char_count).rev() {
        if i < raw_chars.len() {
            cmd.chars.push(raw_chars[i]);
        }
    }
    cmd
}

/// Reusable buffer injector to avoid dynamic allocations during keystroke edits.
#[derive(Debug, Default)]
pub struct BufferInjector {
    pub current_command: EditCommand,
}

impl BufferInjector {
    pub fn new() -> Self {
        Self {
            current_command: EditCommand::default(),
        }
    }

    /// Prepares the internal edit command buffer atomically without reallocations.
    pub fn prepare(&mut self, backspace_count: usize, raw_chars: &[u32], char_count: usize) -> &EditCommand {
        self.current_command.clear();
        self.current_command.backspace_count = backspace_count;
        if self.current_command.chars.capacity() < char_count {
            self.current_command.chars.reserve(char_count - self.current_command.chars.capacity());
        }
        for i in (0..char_count).rev() {
            if i < raw_chars.len() {
                self.current_command.chars.push(raw_chars[i]);
            }
        }
        &self.current_command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_edit_command() {
        // Engine char_data stores output in reverse order
        let raw_chars = vec!['n' as u32, 'â' as u32, 'V' as u32];
        let cmd = generate_edit_command(2, &raw_chars, 3);

        assert_eq!(cmd.backspace_count, 2);
        assert_eq!(cmd.chars, vec!['V' as u32, 'â' as u32, 'n' as u32]);
        assert_eq!(cmd.to_string(), "Vân");
    }

    #[test]
    fn test_buffer_injector_reuse() {
        let mut injector = BufferInjector::new();
        let raw = vec!['a' as u32];
        let cmd1 = injector.prepare(1, &raw, 1);
        assert_eq!(cmd1.backspace_count, 1);
        assert_eq!(cmd1.to_string(), "a");

        let raw2 = vec!['b' as u32, 'a' as u32];
        let cmd2 = injector.prepare(0, &raw2, 2);
        assert_eq!(cmd2.backspace_count, 0);
        assert_eq!(cmd2.to_string(), "ab");
    }
}
