use crate::language::Language;
use crate::history::History;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct FF {
    source_language: HashMap<String, Vec<Vec<String>>>,
    first: HashMap<String, Vec<String>>,
    follow: HashMap<String, Vec<String>>,
    lg: Language,
    table: Table,
    history: History,
}

impl FF {
    pub fn new(lg: Language) -> Self {
        let mut first: HashMap<String, Vec<String>> = HashMap::new();
        let mut follow: HashMap<String, Vec<String>> = HashMap::new();
        Self::gen_first(&mut first, &lg);
        Self::gen_follow(&first, &mut follow, &lg);
        let mut ff = FF {
            source_language: lg.lg.clone(),
            first: first,
            follow: follow,
            lg: lg.clone(),
            table: Table::new(lg.twords.clone(), lg.ntwords.clone()),
            history: History::new()
        };
        ff.build_table();
        ff
    }

    pub fn analyze(&mut self, sentence: String) -> String {
        let mut words: Vec<String> = Vec::new();
        let mut temp = String::new();
        let status;
        for c in sentence.chars() {
            temp.push(c);
            if self.lg.words.contains(&temp) {
                words.push(temp.clone());
                temp.clear();
            }
        } //将语句拆分为单词
        drop(temp); //清理temp

        let eof = "#".to_owned();
        let mut word_iter = words.iter();
        let mut stack: Vec<String> = vec!["#".to_owned(), self.lg.first_word.clone()];
        let mut f = stack.last().unwrap().clone();
        let focus = &mut f;
        let mut word = match word_iter.next() {
            Some(v) => v,
            None => &eof,
        };

        loop {
            if focus == &eof && word == &eof {
                status = "Succeed".to_owned();
                break;
            } else if self.lg.twords.contains(focus) || focus == &eof {
                if focus == word {
                    stack.pop();
                    word = match word_iter.next() {
                        Some(v) => v,
                        None => &eof,
                    }
                } else {
                    status = format!("Error when looking {}", focus);
                    break;
                }
            } else {
                let a = focus.clone();
                let b = word.clone();
                let data = self.table[&(a, b)].clone();
                if data.0 == focus.clone() {
                    stack.pop();
                    self.history.log(&word, &data);
                    for w in data.1.iter().rev() {
                        if *w != "nil".to_owned() {
                            stack.push(w.clone());
                        }
                    }
                } else {
                    status = format!("Error when looking {}", focus);
                    break;
                }
            }
            *focus = stack.last().unwrap().clone();
        }
        println!("{}", self.history);
        status
    }

    fn build_table(&mut self) {
        for entity in self.lg.lg.clone().iter() {
            for v in entity.1 {
                //对文法进行扫描

                let first: Vec<String>;

                if self.lg.twords.contains(&v[0]) {
                    first = vec![v[0].clone()];
                } else {
                    first = self.first.get(&v[0]).unwrap().clone();
                } //取得alpha的first集

                for f in &first {
                    self.table
                        .insert((entity.0.clone(), f.clone()), (entity.0.clone(), v.clone()));
                } //将a belongs to first(alpha) 的 Table[A, a]设置为对应的文法

                if first.contains(&"nil".to_owned()) {
                    let follow = self.follow.get(entity.0).unwrap();
                    for f in follow {
                        self.table
                            .insert((entity.0.clone(), f.clone()), (entity.0.clone(), v.clone()));
                    }
                }
            }
        }
    }

    fn gen_first(first: &mut HashMap<String, Vec<String>>, lg: &Language) {
        let lang = lg.lg.clone();
        for entity in lang.into_iter() {
            let mut word: Vec<String> = Vec::new();
            for v in entity.1 {
                if lg.twords.contains(&v[0]) {
                    word.push(v[0].clone());
                }
            }
            if !first.contains_key(&entity.0) {
                first.insert(entity.0, word);
            } else {
                let w = first.get_mut(&entity.0).unwrap();
                word.extend(w.clone());
                drop(w);
                first.insert(entity.0, word);
            } //第一遍扫描，确认每项所对应的终结符。
        }

        let mut changed = true;
        while changed {
            changed = false;
            let lang = lg.lg.clone();
            for word in lg.ntwords.clone() {
                let mut word_first = match first.get_mut(&word) {
                    None => continue,
                    Some(v) => v.clone(),
                };
                let right = lang.get(&word).unwrap();
                for r in right {
                    if lg.ntwords.contains(&r[0]) {
                        for w in first.get(&r[0]).unwrap().clone() {
                            if !word_first.contains(&w) {
                                word_first.push(w);
                                changed = true;
                            }
                        }
                        first.insert(word.clone(), word_first.clone());
                    } //当右侧第一个为非终结符的时候放入
                }
            }
        }
    }

    fn gen_follow(
        first: &HashMap<String, Vec<String>>,
        follow: &mut HashMap<String, Vec<String>>,
        lg: &Language,
    ) {
        for word in lg.ntwords.clone() {
            follow.insert(word, vec!["#".to_owned()]);
        } //建立HashMap

        let mut changed = true;

        while changed {
            changed = false;
            for word in lg.lg.clone() {
                for r in word.1 {
                    //判断每一个生成式的情况
                    for i in 0..r.len() - 1 {
                        //判断每一个生成式的word
                        if follow.contains_key(&r[i]) {
                            //如果r[i]是非终止符(即需要更新)
                            let mut f = follow.get_mut(&r[i]).unwrap().clone();
                            match first.get(&r[i + 1]) {
                                Some(v) => {
                                    for w in v {
                                        if !f.contains(w) && *w != "nil".to_owned() {
                                            f.push(w.clone()); //将对应的first集放入
                                            changed = true;
                                        }
                                    }
                                    if v.contains(&"nil".to_owned()) {
                                        let f1 = follow.get(&r[i + 1]).unwrap().clone();
                                        for w in f1 {
                                            if !f.contains(&w) {
                                                f.push(w.clone());
                                                changed = true;
                                            }
                                        }
                                    }
                                } //更新Follow集
                                None => {
                                    if !f.contains(&r[i + 1].clone()) {
                                        f.push(r[i + 1].clone());
                                        changed = true;
                                    }
                                } //Do Nothing
                            }
                            follow.insert(r[i].clone(), f.clone());
                        }
                    }

                    if follow.contains_key(&r[r.len() - 1]) {
                        // A -> aB, Follow(B) += Follow(A)
                        let mut f = follow.get_mut(&r[r.len() - 1]).unwrap().clone();
                        let f1 = follow.get(&word.0).unwrap().clone();
                        for w in f1 {
                            if !f.contains(&w) {
                                f.push(w.clone());
                                changed = true;
                            }
                        }
                        follow.insert(r[r.len() - 1].clone(), f);
                    }
                }
            }
        }
    }
}

impl Display for FF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "First collection is:\n")?;
        let first_iter = self.first.iter();
        for w in first_iter{
            write!(f, "{}\t{:?}\n", w.0, w.1)?;
        }
        write!(f, "\n\nFollow collection is:\n")?;
        let follow_iter = self.follow.iter();
        for w in follow_iter{
            write!(f, "{}\t{:?}\n", w.0, w.1)?;
        }
        write!(f, "\n\nTable is:\n{}", self.table)?;
        write!(f, "")
    }
}
#[derive(Debug, Clone)]
struct Table {
    //表结构
    col_index: Vec<String>,
    row_index: Vec<String>,
    data: HashMap<(String, String), (String, Vec<String>)>,
    err: (String, Vec<String>),
} //行：非终结符， 列：终结符， data：推断语句

impl Table {
    fn new(col_index: Vec<String>, row_index: Vec<String>) -> Self {
        let mut data: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
        let err = ("Err".to_owned(), Vec::new()); //将整个表初始化为Err
        let mut col_index = col_index.clone();
        col_index.remove(col_index.len() - 1);
        col_index.push("#".to_owned());
        for c in col_index.clone() {
            for r in row_index.clone() {
                data.insert((r.clone(), c.clone()), err.clone());
            }
        }
        Table {
            col_index,
            row_index,
            data,
            err,
        }
    } //初始化推断表

    fn insert(&mut self, key: (String, String), v: (String, Vec<String>)) {
        self.data.insert(key, v); //在对应位置插入推断语句
    }
}

impl Index<&(String, String)> for Table {
    type Output = (String, Vec<String>);

    fn index(&self, key: &(String, String)) -> &Self::Output {
        match self.data.get(key) {
            Some(v) => v,
            None => &self.err,
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "***\t")?;
        for c in &self.col_index {
            write!(f, "{}\t", *c)?;
        }
        write!(f, "\n")?;

        for r in &self.row_index {
            write!(f, "{}\t", *r)?;
            for c in &self.col_index {
                let data = self.data.get(&(r.clone(), c.clone())).unwrap().clone();
                write!(f, "{}", data.0)?;
                if data.0 != "Err".to_owned() {
                    write!(f, "->")?;
                }
                for w in data.1 {
                    write!(f, "{}", w)?;
                }
                write!(f, "\t")?;
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")
    }
}
