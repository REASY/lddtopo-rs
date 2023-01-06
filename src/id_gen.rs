use std::collections::HashMap;

pub struct IdGen<'a> {
    next_id: u32,
    id_to_str: HashMap<u32, &'a str>,
    str_to_id: HashMap<&'a str, u32>,
}

impl<'a> IdGen<'a> {
    pub fn new() -> IdGen<'a> {
        IdGen {
            next_id: 0,
            id_to_str: HashMap::new(),
            str_to_id: HashMap::new(),
        }
    }

    pub fn get_next_id(&mut self, str: &'a str) -> u32 {
        let id = match self.str_to_id.get(str) {
            None => {
                let id = self.next_id;
                self.str_to_id.insert(str, id);
                self.id_to_str.insert(id, str);
                self.next_id += 1;
                id
            }
            Some(id) => { *id }
        };
        id
    }

    pub fn get_by_id(&self, id: u32) -> Option<&'a str> {
        self.id_to_str.get(&id).map(|r| { *r })
    }
}