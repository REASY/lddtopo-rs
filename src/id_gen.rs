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
                assert_ne!(id, u32::MAX, "Reached u32::MAX");
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

#[cfg(test)]
pub(crate) mod tests {
    use crate::id_gen::IdGen;

    #[test]
    fn new_works() {
        let id_gen = IdGen::new();
        assert_eq!(0, id_gen.next_id);
        assert!(id_gen.id_to_str.is_empty());
        assert!(id_gen.str_to_id.is_empty());
    }

    #[test]
    fn get_next_id_when_the_input_is_the_same_should_return_the_same_id() {
        let mut id_gen = IdGen::new();
        let str = "hello";
        let id = id_gen.get_next_id(str);
        assert_eq!(0, id);
        assert_eq!(0, id_gen.get_next_id(str));
        assert_eq!(0, id_gen.get_next_id(str));
    }

    #[test]
    fn get_next_id_when_the_input_is_differt_should_return_new_id() {
        let mut id_gen = IdGen::new();
        let s1 = "hello";
        let s2 = "ello";
        let id1 = id_gen.get_next_id(s1);
        let id2 = id_gen.get_next_id(s2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn get_by_id_when_id_does_not_exist_should_return_none() {
        let id_gen = IdGen::new();
        let id = id_gen.get_by_id(12);
        assert!(id.is_none());
    }

    #[test]
    fn get_by_id_when_id_exists_should_return_some() {
        let mut id_gen = IdGen::new();
        let str = "hello";
        let id = id_gen.get_next_id(str);
        match id_gen.get_by_id(id) {
            None => {
                panic!("expected to get str")
            }
            Some(s) => {
                assert_eq!(str, s);
            }
        };
    }
}