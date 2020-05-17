use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct History {
    step: usize,
    rule: Vec<String>,
    action: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        History {
            step: 0,
            rule: Vec::new(),
            action: Vec::new(),
        }
    }

    pub fn log(&mut self, word: &String, rule: &(String, Vec<String>)) {
        let mut w = word.clone();
        w = "Anazying ".to_owned() + w.as_str();
        self.action.push(w);
        let r = rule.clone();
        let mut ru = format!("{}->", r.0);
        for a in r.1 {
            ru = format!("{}{}", ru, a);
        }
        self.rule.push(ru);
        self.step += 1;
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "step\taction\trule\n")?;
        for i in 0..self.step{
            write!(f,"{}\t{}\t{}\n", i, self.action[i], self.rule[i])?;
        }
        write!(f, "")
    }
}
