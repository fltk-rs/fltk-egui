pub struct Clipboard {
    clipboard: Option<copypasta::ClipboardContext>,
}


impl Default for Clipboard {
    fn default() -> Self {
        Self {
            clipboard: init_copypasta(),
        }
    }
   
}

impl Clipboard {
    pub fn get(&mut self) -> Option<String> {
        if let Some(clipboard) = &mut self.clipboard {
            use copypasta::ClipboardProvider as _;
            match clipboard.get_contents() {
                Ok(contents) => Some(contents),
                Err(err) => {
                    eprintln!("Paste error: {}", err);
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn set(&mut self, text: String) {
        if let Some(clipboard) = &mut self.clipboard {
            use copypasta::ClipboardProvider as _;
            if let Err(err) = clipboard.set_contents(text) {
                eprintln!("Copy/Cut error: {}", err);
            }
        }
    }

}

fn init_copypasta() -> Option<copypasta::ClipboardContext> {
    match copypasta::ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}