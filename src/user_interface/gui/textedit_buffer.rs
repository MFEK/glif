use std::collections::HashMap;

pub trait EditBuffer {
    fn reset(&mut self, id: impl Into<String>);
    fn set_buf(&mut self, id: impl Into<String>, v: String);
    fn get_buf(&mut self, id: impl Into<String>, default: &String) -> &mut String;
}

impl EditBuffer for HashMap<String, String> {
    fn get_buf(&mut self, id: impl Into<String>, default: &String) -> &mut String {
        let id: &String = &id.into();
        if self.get_mut(id).is_none() {
            self.insert(id.into(), default.clone());
        }

        self.get_mut(id).unwrap()
    }

    fn reset(&mut self, id: impl Into<String>) {
        self.remove(&id.into());
    }

    fn set_buf(&mut self, id: impl Into<String>, v: String) {
        self.insert(id.into(), v);
    }
}

