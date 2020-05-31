use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::iter::Extend;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Language {
    pub twords: Vec<String>,
    pub ntwords: Vec<String>,
    pub words: Vec<String>,
    pub lg: HashMap<String, Vec<Vec<String>>>,
    conf_path: String,
    pub first_word: String,
}

impl Language {
    pub fn start() -> Language {
        Language {
            twords: Vec::new(),
            ntwords: Vec::new(),
            words: Vec::new(),
            lg: HashMap::new(),
            conf_path: String::new(),
            first_word: String::new(),
        }
    }

    pub fn new_from_string(&mut self, conf: String, nt: String, t: String, lr:bool) {
        self.gen_t(t);
        self.gen_nt(nt);
        let mut words = Vec::new();
        words.append(&mut self.ntwords.clone());
        words.append(&mut self.twords.clone());
        self.words = words;
        self.gen_lg(conf);
        if !lr {
            self.left_re();
        }
    }

    pub fn new(&mut self, conf_path: &str, tpath: &str, ntpath: &str, lr:bool) {
        let words_string = Self::read_word_file(tpath.to_owned());
        self.gen_t(words_string);
        let words_string = Self::read_word_file(ntpath.to_owned());
        self.gen_nt(words_string);

        let mut words = Vec::new();
        words.append(&mut self.ntwords.clone());
        words.append(&mut self.twords.clone());
        self.words = words;

        self.conf_path = String::from(conf_path);
        let path = Path::new(&self.conf_path);
        let display = path.display();

        let conf_file = match File::open(path) {
            Err(why) => {
                panic!("cannot open file {}, because {}\n", display, why);
            }
            Ok(file) => {
                print!("conf file opened succeed\n");
                file
            }
        }; //打开语法描述文件

        let mut lines = io::BufReader::new(conf_file);
        let mut f = String::new();
        match lines.read_to_string(&mut f) {
            Ok(_) => {}
            Err(e) => println!("Error is {}", e),
        }
        self.gen_lg(f);
        if !lr{
            self.left_re();
        }
    }

    fn gen_nt(&mut self, words_string: String) {
        let words: Vec<&str> = words_string.split(' ').collect();
        self.ntwords = words.iter().map(|&x| x.to_owned()).collect(); //获得词表
    }

    fn gen_t(&mut self, words_string: String) {
        let words: Vec<&str> = words_string.split(' ').collect();
        self.twords = words.iter().map(|&x| x.to_owned()).collect(); //获得词表
    }

    fn gen_lg(&mut self, lines: String) {
        let lines = lines.split("\n");
        let mut first = true;
        for line in lines {
            let l = line; // 解包line

            let words: Vec<&str> = l.split(' ').collect();
            let key = words[0].to_owned();
            if first {
                self.first_word = words[0].to_owned();
                first = false;
            }
            let v: Vec<&str> = words[1].split('|').collect(); //得到生成式右侧
            let mut r: Vec<Vec<String>> = Vec::new();
            for s in v {
                let mut gen: Vec<String> = Vec::new();
                let s = s.to_owned();
                let mut t = String::new();
                for c in s.chars() {
                    if self.words.contains(&t) {
                        gen.push(t.clone());
                        t.clear();
                        t.push(c);
                    } else {
                        t.push(c);
                    }
                }
                if self.words.contains(&t) {
                    gen.push(t.clone());
                }
                if gen.len() > 0 {
                    r.push(gen);
                }
            }
            if self.lg.contains_key(&key) {
                let source = self.lg.get(&key).unwrap().clone();
                r.extend(source);
                self.lg.insert(key, r);
            } else {
                self.lg.insert(key, r);
            }
        } //将数据放入
    }

    fn left_re(&mut self) {
        //直接左递归消除程序
        let mut changed = true;
        while changed {
            changed = false;
            for entity in self.lg.clone() {
                let mut entity = (entity.0.clone(), entity.1.clone());
                let mut alpha: Vec<usize> = Vec::new();
                let mut beta: Vec<usize> = Vec::new();

                for i in 0..entity.1.len() {
                    if entity.1[i][0] == entity.0.clone() {
                        //当产生式右部的最左端的符号和产生式左端符号相同的时候，检查到直接左递归
                        //alpha即为存在左公因式的产生式右端
                        alpha.push(i);
                    } else {
                        beta.push(i);
                    } //确定alpha和beta，便于提取左公因式
                }

                if !alpha.is_empty() {
                    changed = true;
                    let new_left = "!".to_owned() + entity.0.clone().as_str(); //建立新的产生式左部(A->!A)
                    for s in beta {
                        entity.1[s].push(new_left.clone());
                    } //将新的产生式添加到右部
                    alpha.sort();
                    let mut new_gen: Vec<Vec<String>> = Vec::new();
                    let mut f = 0;
                    for s in alpha {
                        entity.1[s].remove(0); //去除所有的左公因式
                        new_gen.push(entity.1[s].clone()); //将消除了公因式的产生式放到新的产生式中(alpha)
                        entity.1.remove(s - f); //将已经添加到新的产生式中的被提取了公因式的项删除
                        f += 1;
                    }
                    for g in &mut new_gen{
                        g.push(new_left.clone());
                    }
                    new_gen.push(vec!["nil".to_owned()]);
                    self.ntwords.push(new_left.clone());
                    self.words.push(new_left.clone());
                    self.lg.insert(entity.0, entity.1);
                    self.lg.insert(new_left, new_gen);
                }
            }
        }
    }

    fn read_word_file(path: String) -> String {
        let w_path = Path::new(path.as_str()); //读取词表
        let w_display = w_path.display();

        let mut word_file = match File::open(w_path) {
            Err(why) => panic!("cannot open file {}, because {}\n", w_display, why),
            Ok(file) => file,
        };

        let mut words_string = String::new();
        match word_file.read_to_string(&mut words_string) {
            Err(why) => panic!(
                "error while read NTword file {}, because of {}\n",
                w_display, why
            ),
            Ok(_) => {}
        }
        words_string
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "terminal words: {:?}\n", self.twords)?;
        write!(f, "non-terminal words: {:?}\n", self.ntwords)?;
        for entity in &self.lg{
            write!(f, "{}->", entity.0)?;
            for right in entity.1{
                write!(f, "\t")?;
                for words in right{
                    write!(f, "{}", words)?;
                }
                write!(f, "\n")?;
            }
        }
        write!(f, "")
    }
}
