use crate::ff::FF;
use crate::language::Language;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct LR {
    pub ll: FF,                          //LL1语法分析器(提供First集)
    lang: Language,                      //已解析的语法
    cc:Vec<Vec<Gen>>,
    transation: HashMap<(usize, String), usize>,
    action: HashMap<(usize, String), String>,
    goto: HashMap<(usize, String), usize>,
    history: History
}

impl LR {
    pub fn new(lang: &Language) -> Self {
        let mut lang = lang.clone();
        
        let first = lang.first_word.clone();
        let new_first = "\'".to_owned() + first.clone().as_str(); //假设开头的非终结符是S，则会放入'S作为新的开头
        lang.lg.insert(new_first.clone(), vec![vec![first]]);
        lang.first_word = new_first.clone(); //对文法进行扩增
        lang.ntwords.push(new_first.clone());
        lang.words.push(new_first);
        let ll = FF::new(&lang); //从扩增文法中计算出First和Follow(只需要First)
        let mut lr = LR {
            ll: ll.clone(),
            lang: lang,
            cc: Vec::new(),
            transation: HashMap::new(),
            action: HashMap::new(),
            goto: HashMap::new(),
            history: History::new()
        };
        lr.cc();
        lr.fill_table();
        lr
    }

    fn get_first(&self, word: String) -> Vec<String> {
        if self.lang.twords.contains(&word) || word == "#".to_owned() {
            if word == "nil".to_owned(){
                vec!["#".to_owned()]
            }else{
                vec![word.clone()]
            }
            
        } else if self.lang.ntwords.contains(&word) {
            self.ll.first.get(&word).unwrap().clone()
        } else {
            panic!("Error when get first collection")
        }
    }

    pub fn get_history(&self) -> String{
        format!("{}", self.history)
    }

    fn closure(&self, s:&mut Vec<Gen>){
        let mut changed = true;
        while changed {
            changed = false;
            for g in s.clone(){
                let word = g.get_current_place();
                for p in self.lang.lg.clone(){
                    let first;
                    if g.get_next_place() == "#".to_owned() {
                        first = self.get_first(g.ahead.clone());
                    } else {
                        first = self.get_first(g.get_next_place());
                    } //获取first集(如果生成式到达了末尾，则使用ahead来生成first集)
                    for w in first{
                        if p.0 == word{
                            for r in &p.1{
                                if w == "nil".to_owned(){
                                    continue;
                                }
                                let gen = Gen::new(&(word.clone(), r.clone()), Some(w.clone()), 0);
                                if !s.contains(&gen){
                                    s.push(gen);
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn goto(&self, s: &Vec<Gen>, x:&String) -> Vec<Gen>{
        let mut moved = Vec::new();
        for g in s{
            let curr = g.get_current_place();
            if curr == *x{
                let mut gen = g.clone();
                gen.move_to_next();
                if !moved.contains(&gen){
                    moved.push(gen);
                }
            }
        }
        self.closure(&mut moved);
        moved
    }

    fn cc(&mut self){
        let first_word = self.lang.first_word.clone();
        let mut cc0 = Vec::new();
        let pro = self.lang.lg.get(&first_word).unwrap().clone();
        let pro = pro[0].clone();
        let gen = Gen::new(&(first_word, pro), None, 0);
        cc0.push(gen);
        self.closure(&mut cc0);
        self.cc.push(cc0);
        let mut marked = vec![false];
        let mut changed = true;
        while changed{
            changed = false;
            for i in 0..marked.len(){
                if !marked[i]{
                    marked[i] = !marked[i];
                    let c = self.cc[i].clone();
                    
                    for g in c.clone(){
                        let mut temp= Vec::new();
                        let x = g.get_current_place();
                        temp.extend(self.goto(&c, &x));
                        if !self.cc.contains(&temp){
                            self.cc.push(temp);
                            marked.push(false);
                            self.transation.insert((i, x),  marked.len() - 1);
                            changed = true;
                        }else{
                            for s in 0..self.cc.len(){
                                if temp == self.cc[s]{
                                    match self.transation.insert((i, x), s){
                                        Some(_) => {},
                                        None => changed = true
                                    };
                                    break;
                                }
                            }
                            
                        }
                    }
                    
                }
            }
        }
    }

    fn fill_table(&mut self){
        for ccn in 0..self.cc.len(){
            for g in &self.cc[ccn]{
                if g.len > g.p {
                    let next = match self.transation.get(&(ccn, g.get_current_place())){
                        Some(v) => v,
                        None => {panic!("Error When building action table")}
                    };
                    let curr = g.get_current_place();
                    if self.lang.twords.contains(&curr){
                        self.action.insert((ccn, g.get_current_place()), format!("s{}", next));
                    }else{
                        self.goto.insert((ccn, g.get_current_place()), *next);
                    }
                }else if g.len == g.p && !(g.gen.0 == self.lang.first_word && g.ahead == "#".to_owned()){
                    let mut index = 0;
                    let left = &g.gen.0;
                    for l in &self.lang.lg{
                        if l.0 == left{
                            for r in 0..l.1.len(){
                                if l.1[r] == g.gen.1{
                                    index = r;
                                }
                            }
                        }
                    }
                    self.action.insert((ccn, g.get_current_place()), format!("r:{}-{}", left, index));//获得规约所使用的产生式
                }else if g.gen.0 == self.lang.first_word && g.ahead == "#".to_owned(){
                    self.action.insert((ccn, "#".to_owned()), "acc".to_owned());
                }
            }
            for w in &self.lang.ntwords{
                match self.transation.get(&(ccn, w.clone())) {
                    Some(v) =>{
                        self.goto.insert((ccn, w.clone()), *v);
                    },
                    None => {}
                }
            }
        }
    }

    pub fn analyze(&mut self, sentence: String) -> String {
        let mut word_stack: Vec<String> = Vec::new();
        let mut status_stack: Vec<usize> = Vec::new();
        let mut sen: Vec<String> = Vec::new();
        let mut temp = String::new();

        for c in sentence.chars() {
            temp.push(c);
            if self.lang.words.contains(&temp) {
                sen.push(temp.clone());
                temp.clear();
            }
        }
        if !temp.is_empty() {
            sen.push(temp.clone());
            drop(temp);
        } //将输入语句拆分为单词
        sen.push("#".to_owned());//完成对输入串的预处理

        word_stack.push("#".to_owned());
        status_stack.push(0);//双栈的初始化完成

        
        let reg: Regex = Regex::new(r"('|\w)+-(\d)").unwrap();
        let shift = Regex::new(r"s(\d)+").unwrap();
        let num: Regex = Regex::new(r"([0-9])+").unwrap();
        let gen_: Regex = Regex::new(r"('|[A-Z])+").unwrap();

        let mut i = 0;
        loop{
            let curr = status_stack[status_stack.len()-1];
            self.history.log(word_stack.clone(), status_stack.clone(), sen[i..sen.len()].to_vec());
            let next = match self.action.get(&(curr, sen[i].clone())) {
                Some(v) => {
                    v.clone()
                },
                None => {
                    "Err".to_owned()
                }
            };
            if next == "Err".to_owned(){
                return format!("Fail to analyze when looking {}", sen[i]);
            }else{
                if reg.is_match(next.as_str()){//情况分支，规约
                    let w = gen_.captures(next.as_str()).unwrap().get(1).unwrap().as_str().to_owned();
                    let reg_num = num.captures(next.as_str()).unwrap().get(1).unwrap().as_str().to_owned().parse::<usize>().unwrap();
                    let reg = self.lang.lg.get(&w).unwrap()[reg_num].clone();
                    for _ in 0..reg.len(){
                        word_stack.pop();
                        status_stack.pop();
                    }//删除和规约串长度相等数量的字符
                    word_stack.push(w.clone());
                    let curr_status = status_stack[status_stack.len()-1];
                    let next_status = match self.goto.get(&(curr_status, w.clone())) {
                        Some(v) => v,
                        None => {
                            println!("{}", self.history);
                            panic!("Error when looking {}", w)
                        },
                    };
                    status_stack.push(next_status.clone()); 
                    self.history.log(word_stack.clone(), status_stack.clone(), sen[i..sen.len()].to_vec());
                }else if next == "acc".to_owned(){
                    break;
                }else if shift.is_match(next.as_str()){//情况分支：移进
                    word_stack.push(sen[i].clone());
                    status_stack.push(num.captures(next.as_str()).unwrap().get(1).unwrap().as_str().to_owned().parse::<usize>().unwrap());
                    i += 1;
                    self.history.log(word_stack.clone(), status_stack.clone(), sen[i..sen.len()].to_vec());
                }else{
                    panic!("Fatal Error");
                }
            }
        }
        "Success".to_owned()
    }
}

impl Display for LR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}\n", self.lang)?;
        write!(f, "{}\n", self.ll)?;
        for c in 0..self.cc.len(){
            write!(f, "cc{}\n", c)?;
            for g in &self.cc[c]{
                write!(f, "{}\n", g)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "\n\t")?;
        for w in &self.lang.twords{
            if w == &"nil".to_owned(){
                continue;
            }
            write!(f, "{}\t", w)?;
        }
        write!(f, "#\t")?;
        for w in &self.lang.ntwords{
            write!(f, "{}\t", w)?;
        }
        write!(f, "\n")?;
        
        for i in 0..self.cc.len(){
            write!(f, "{}\t", i)?;
            for w in &self.lang.twords{
                if w == &"nil".to_owned(){
                    continue;
                }
                match self.action.get(&(i, w.clone())) {
                    Some(v) => write!(f, "{}\t", v),
                    None => write!(f, "e\t")
                }?;
            }

            match self.action.get(&(i, "#".to_owned())) {
                Some(v) => write!(f, "{}\t", v),
                None => write!(f, "e\t")
            }?;
    
            for w in &self.lang.ntwords{
                match self.goto.get(&(i, w.clone())) {
                    Some(v) => write!(f, "{}\t", v),
                    None => write!(f, "e\t")
                }?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

#[derive(Clone, Debug)]
struct Gen {
    pub gen: (String, Vec<String>), //原始生成式(单句)
    pub ahead: String,              //前瞻符号,
    pub p: usize,                   //语法分析器栈顶位置
    pub len: usize,
}

impl Gen {
    pub fn new(source: &(String, Vec<String>), ahead: Option<String>, p: usize) -> Self {
        Gen {
            gen: source.clone(),
            ahead: match ahead {
                Some(v) => v,
                None => "#".to_owned(),
            },
            p: p,
            len: source.1.len(),
        }
    }

    pub fn get_current_place(&self) -> String {
        if self.p < self.gen.1.len() {
            self.gen.1[self.p].clone()
        } else {
            "#".to_owned()
        }
    } //返回当前栈顶的字符

    pub fn get_next_place(&self) -> String {
        if self.p + 1 < self.gen.1.len() {
            self.gen.1[self.p + 1].clone()
        } else {
            "#".to_owned()
        }
    } //返回预期放入栈顶的字符

    pub fn move_to_next(&mut self) -> bool {
        if self.p < self.len {
            self.p += 1;
            true
        } else {
            false //若已经到生成式末尾，无法移动
        }
    } //将栈顶符号后移一位
}

impl PartialEq for Gen {
    fn eq(&self, another: &Gen) -> bool {
        self.ahead == another.ahead
            && self.len == another.len
            && self.gen == another.gen
            && self.p == another.p
    }
}

impl Display for Gen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{} ->", self.gen.0)?; //输出产生式右边
        for i in 0..self.len + 1 {
            if i < self.len {
                if i == self.p {
                    write!(f, "·{}", self.gen.1[i])?;
                } else {
                    write!(f, "{}", self.gen.1[i])?;
                }
            } else {
                if self.p == i {
                    write!(f, "·")?;
                }
            }
        }
        write!(f, " , {}", self.ahead)
    }
}


#[derive(Clone, Debug)]
struct History {
    len: usize,
    step: Vec<String>,
    input: Vec<String>
}

impl History {
    pub fn new() -> Self {
        Self {
            len: 0,
            step: Vec::new(),
            input: Vec::new()
        }
    }

    pub fn log(&mut self, word_stack: Vec<String>, status_stack: Vec<usize>, input: Vec<String>) {
        self.len += 1;
        let mut ws = String::new();
        for w in word_stack{
            ws = format!("{} {}", ws, w);
        }
        let mut ss = String::new();
        for w in status_stack{
            ss = format!("{} {}", ss, w);
        }
        self.step.push(format!("{}\t{}", ws, ss));
        let mut i = String::new();
        for w in input{
            i = format!("{} {}", i, w);
        }
        self.input.push(i);
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "\tword_stack\tstatus_stack\tinput\n")?;
        for i in 0..self.step.len(){
            write!(f, "{}\t{}\t{}\n", i, self.step[i], self.input[i])?;
        }
        write!(f, "")
    }
}
