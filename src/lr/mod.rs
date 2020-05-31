use crate::ff::FF;
use crate::language::Language;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct LR {
    pub ll: FF,                          //LL1语法分析器(提供First集)
    lang: Language,                      //已解析的语法
    gen_list: Vec<(String, usize, Gen)>, //改写为LR1语法元组的生成式(左式，点位置，产生式)
    closeure: Vec<Vec<Gen>>,             //闭包
    index: Vec<(String, usize)>,
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
            gen_list: Vec::new(),
            closeure: Vec::new(),
            index: Vec::new(),
        };
        lr.gen_genlist(); //生成所有的LR0式的生成式
        lr.gen_clousre(); //建立所有的闭包
        lr
    }

    fn gen_genlist(&mut self) {
        for entity in self.lang.lg.clone() {//从原语言配置中读取所有的产生式
            for right in entity.1.clone() {
                let gen = (entity.clone().0.clone(), right.clone());
                if !gen.1.contains(&"nil".to_owned()){
                    for p in 0..gen.1.len() + 1 {
                        let g = Gen::new(&gen, None, p);
                        self.gen_list.push((entity.clone().0, p, g.clone()));
                        self.index.push((entity.0.clone(), p));
                        for word in self.lang.words.clone() {
                            if word != "nil".to_owned() && gen.0 != self.lang.first_word {
                                let g = Gen::new(&gen, Some(word), p);
                                self.gen_list.push((entity.clone().0, p, g.clone()));
                                self.index.push((entity.0.clone(), p));
                            }
                        }
                    }//非空串的时候，原样放入
                }else{
                    let mut gen = gen.clone();
                    gen.1.clear();
                    let g = Gen::new(&gen, None, 0);
                    self.index.push((entity.clone().0, 0));
                    self.gen_list.push((entity.clone().0, 0, g.clone()));
                    for word in self.lang.words.clone() {
                        if word != "nil".to_owned() && gen.0 != self.lang.first_word {
                            let g = Gen::new(&gen, Some(word), 0);
                            self.gen_list.push((entity.clone().0, 0, g.clone()));
                            self.index.push((entity.0.clone(), 0));
                        }
                    }
                }
            }
        }
    }

    fn gen_clousre(&mut self) {
        for g in self.gen_list.clone() {
            let mut closure: Vec<Gen> = Vec::new(); //单个闭包
            let gen = g.2.clone();
            closure.push(gen.clone()); //将第一个先放入闭包
            let curr = gen.get_current_place(); //获得当前栈顶的符号，判断是否需要进一步扩展
            if self.lang.ntwords.contains(&curr) {
                //如果是非终结符(开始扩展)
                //将所有(curr, 0, gen)取出，并添加到闭包中
                let first;
                if gen.get_next_place() == "#".to_owned() {
                    first = self.get_first(gen.ahead.clone());
                } else {
                    first = self.get_first(gen.get_next_place());
                } //获取first集(如果生成式到达了末尾，则使用ahead来生成first集)

                let mut index = Vec::new();
                for i in 0..self.index.clone().len() {
                    if self.index[i] == (curr.clone(), 0) && !index.contains(&i) {
                        index.push(i);
                    }
                } //获得所有满足条件的index
                for i in index {
                    let g = self.gen_list[i].clone().2;
                    for f in first.clone() {
                        //使用下一个位置的first集来作为前瞻符号
                        let mut g_t = g.clone();
                        g_t.ahead = f; //将将要放入闭包的生成式的ahead更新为beta a的first集中的某一位
                        if !closure.contains(&g_t) && g_t.ahead != "nil".to_owned() {
                            closure.push(g_t);
                        } //将所有可能的全部放入clousure
                    }
                }
            }
            if !self.closeure.contains(&closure) {
                self.closeure.push(closure);
            } //将完成了第一次扩张的闭包放入闭包列表中
        }

        //进行闭包的完善
        let mut changed = true;
        while changed {
            changed = false;
            let s = self.clone();
            for c in &mut self.closeure {
                for g in c.clone() {
                    let curr = g.clone().get_current_place();
                    if self.lang.ntwords.contains(&curr) {
                        //如果是非终结符(开始扩展)
                        //将所有(curr, 0, gen)取出，并添加到闭包中
                        let first;
                        if g.get_next_place() == "#".to_owned() {
                            first = s.get_first(g.ahead.clone());
                        } else {
                            first = s.get_first(g.get_next_place());
                        } //获取first集(如果生成式到达了末尾，则使用ahead来生成first集)

                        let mut index = Vec::new();
                        for i in 0..self.index.clone().len() {
                            if self.index[i] == (curr.clone(), 0) && !index.contains(&i) {
                                index.push(i);
                            }
                        } //获得所有满足条件的index
                        for i in index {
                            let g = self.gen_list[i].clone().2;
                            for f in first.clone() {
                                //使用下一个位置的first集来作为前瞻符号
                                let mut g_t = g.clone();
                                g_t.ahead = f; //将将要放入闭包的生成式的ahead更新为beta a的first集中的某一位
                                if !c.contains(&g_t) && g_t.ahead != "nil".to_owned() {
                                    c.push(g_t);
                                    changed = true;
                                } //将所有可能的全部放入clousure
                            }
                        }
                    }
                }
            }
        }
        let first = &self.lang.first_word;
        let mut index = self.index.len() + 1;
        for i in 0..self.index.len() {
            if self.index[i] == (first.clone(), 0) {
                index = i;
            }
        }
        let first_gen = self.gen_list[index].clone();
        let mut tc = Vec::new();
        for c in 0..self.closeure.len() {
            if self.closeure[c].contains(&first_gen.2) {
                let mut cc = self.closeure[c].clone();
                cc.remove(0);
                tc = cc.clone();
                break;
            }
        }
        if tc.len() > 0 {
            let len = self.closeure.len();
            for i in 0..len {
                if self.closeure[i] == tc {
                    self.closeure.remove(i);
                    break;
                }
            }
        }
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
}

impl Display for LR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}\n", self.lang)?;
        write!(f, "{}\n", self.ll)?;
        for i in 0..self.closeure.len() {
            write!(f, "closeure{}:\n", i)?;
            for g in self.closeure[i].clone() {
                write!(f, "{}\n", g)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

pub struct AnalyzeTable {
    action: HashMap<(usize, String), usize>,
    goto: HashMap<(usize, String), usize>,
    reg: HashMap<(usize, String), String>,
    lr: LR,
    history: History
}

impl AnalyzeTable {
    pub fn new(lang: &Language) -> Self {
        let mut lr = LR::new(lang);
        let go = GO::new(&mut lr);
        let mut action: HashMap<(usize, String), usize> = HashMap::new();
        let mut goto: HashMap<(usize, String), usize> = HashMap::new();
        let mut reg: HashMap<(usize, String), String> = HashMap::new();
        let mut end = 0;
        for i in 0..lr.index.len() {
            if lr.index[i] == (lr.lang.first_word.clone(), 1) {
                end = i; //获得接受状态对应的序号
            }
        }
        let end_gen = lr.gen_list[end].clone();
        for i in 0..lr.closeure.len() {
            if lr.closeure[i].contains(&end_gen.2) {
                end = i;
            }
        }
        for entity in go.go {
            if lr.lang.twords.contains(&(entity.0).1) {
                if entity.1 == end {
                    action.insert((entity.1, "#".to_owned()), lr.closeure.len());
                }
                action.insert(entity.0, entity.1);
            } else if lr.lang.ntwords.contains(&(entity.0).1) {
                goto.insert(entity.0, entity.1);
            }
        }
        for i in 0..lr.closeure.len() {
            let c = &lr.closeure[i];
            for g in c {
                if g.p == g.len {
                    let gen = g.gen.clone();
                    for sen in &lr.lang.lg {
                        if sen.0 == &gen.0 {
                            for s in 0..sen.1.len() {
                                if sen.1[s] == gen.1 {
                                    if sen.0 != &lr.lang.first_word {
                                        reg.insert(
                                            (i, g.ahead.clone()),
                                            format!("{}-{}", sen.0, s),
                                        );
                                    } else {
                                        reg.insert((i, g.ahead.clone()), "acc".to_owned());
                                    }
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        Self {
            action,
            goto,
            reg,
            lr,
            history: History::new()
        }
    }

    pub fn analyze(&mut self, sentence: String) -> String {
        let mut word_stack: Vec<String> = Vec::new();
        let mut status_stack: Vec<usize> = Vec::new();
        let mut sen: Vec<String> = Vec::new();
        let mut temp = String::new();

        for c in sentence.chars() {
            temp.push(c);
            if self.lr.lang.words.contains(&temp) {
                sen.push(temp.clone());
                temp.clear();
            }
        }
        if !temp.is_empty() {
            sen.push(temp.clone());
            drop(temp);
        } //将输入语句拆分为单词
        sen.push("#".to_owned());//完成对输入串的预处理

        let first_index = (self.lr.lang.first_word.clone(), 0);
        let mut index = 0;
        for g in 0..self.lr.index.len() {
            if self.lr.index[g] == first_index {
                index = g;
            }
        }
        let first_gen = self.lr.gen_list[index].clone().2;
        let mut start = self.lr.closeure.len();
        for i in 0..self.lr.closeure.len() {
            if self.lr.closeure[i].contains(&first_gen) {
                start = i;
            }
        }
        lazy_static! {
            static ref REG: Regex = Regex::new(r"('|\w)+-(\d)").unwrap();
        }
        let num: Regex = Regex::new(r"([0-9])+").unwrap();
        let gen_: Regex = Regex::new(r"('|[A-Z])+").unwrap();
        status_stack.push(start);
        word_stack.push("#".to_owned());
        //let mut history = History::new();
        self.history.log(word_stack.clone(), status_stack.clone(), sen[0..sen.len()].to_vec());
        let mut i = 0;
        loop {
            let word = sen[i].clone();
            if self.lr.lang.words.contains(&word) || word == "#".to_owned() {
                let curr_status = status_stack[status_stack.len() - 1];
                let next_status = match self.action.get(&(curr_status, word.clone())) {
                    Some(v) => v.clone().to_string(),
                    None => match self.reg.get(&(curr_status, word.clone())) {
                        Some(v) => (*v).clone(),
                        None => "e".to_owned(),
                    },
                };
                if next_status == "acc".to_owned() {
                    return "Succeed".to_owned(); //分析成功
                }
                if next_status == "e".to_owned() {
                    return format!("history is:\n{}Error When Processing {}",self.history, word);
                } else if REG.is_match(next_status.clone().as_str()) {
                    //情况分支：规约
                    let reg_stat = num.captures(next_status.as_str()).unwrap();
                    let reg_start = gen_.captures(next_status.as_str()).unwrap();
                    let num = reg_stat.get(1);
                    let num = num.unwrap().as_str().to_owned();
                    let num = num.parse::<usize>().unwrap();
                    let genp = reg_start.get(1).unwrap().as_str().to_owned(); //找到对应的生成式

                    let generate = match self.lr.lang.lg.get(&genp) {
                        Some(v) => (*v).clone()[num].clone(),
                        None => panic!("Cannot find right generate, please check program"),
                    }; //获得对应规约生成式

                    let gen_len = generate.len();
                    for _ in 0..gen_len {
                        status_stack.pop();
                        word_stack.pop();
                    } //将对应数量的符号移出栈

                    let curr_status = status_stack[status_stack.len()-1];
                    let next_status = match self.goto.get(&(curr_status, genp.clone())) {
                        Some(v) => v,
                        None => {
                            println!("{}", self.history);
                            panic!("Error when looking {}", genp)
                        },
                    };
                    status_stack.push(next_status.clone()); //将规约得到的状态放入栈中
                    word_stack.push(genp);
                    self.history.log(word_stack.clone(), status_stack.clone(), sen[i..sen.len()].to_vec());
                } else {
                    //情况分支：移进
                    i += 1;
                    status_stack.push(next_status.parse::<usize>().unwrap());
                    word_stack.push(word.clone());
                    self.history.log(word_stack.clone(), status_stack.clone(), sen[i..sen.len()].to_vec());
                }
            }
        }
    }
    pub fn get_history(&self) -> String{
        format!("{}", self.history)
    }
}

impl Display for AnalyzeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}\n \t", self.lr)?;
        for w in &self.lr.lang.twords {
            write!(f, "{}\t", w)?;
        }
        write!(f, "#\t")?;
        for w in &self.lr.lang.ntwords {
            write!(f, "{}\t", w)?;
        }
        write!(f, "\n")?;
        let mut temp = self.lr.lang.twords.clone();
        temp.push("#".to_owned());
        for i in 0..self.lr.closeure.len() {
            write!(f, "{}\t", i)?;
            for w in &temp {
                match self.action.get(&(i, w.clone())) {
                    Some(v) => write!(f, "s{}\t", v),
                    None => match self.reg.get(&(i, w.clone())) {
                        Some(v) => write!(f, "r:{}\t", v),
                        None => write!(f, "e\t"),
                    },
                }?;
            }
            for w in &self.lr.lang.ntwords {
                match self.goto.get(&(i, w.clone())) {
                    Some(v) => write!(f, "{}\t", v),
                    None => write!(f, "e\t"),
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

struct GO {
    //词表(终结符or非终结符)
    go: HashMap<(usize, String), usize>, //<(闭包序号, 输入字符), 目标闭包序号>
}

impl GO {
    pub fn new(lr: &mut LR) -> Self {
        let mut go = Self { go: HashMap::new() };
        let mut used = Vec::new();
        let mut way: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..lr.closeure.len() {
                let c = lr.closeure[i].clone();
                for g in c {
                    let mut gen = g.clone();
                    if gen.move_to_next() {
                        for s in 0..lr.closeure.len() {
                            if lr.closeure[s].contains(&gen) {
                                if !used.contains(&i){
                                    used.push(i);
                                    changed = true;
                                }
                                if !used.contains(&s){
                                    used.push(s);
                                    changed = true;
                                } //找到goto的方向，填入goto表(其中gen.get_current_place()可以获取移动前待识别的字符)
                                match way.get_mut(&i) {
                                    Some(v) => {
                                        if !v.contains(&s){
                                            v.push(s);
                                            changed = true;
                                        }
                                    },
                                    None => match way.insert(i, vec![s]) {
                                        _ => changed = true
                                    },
                                }
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
            }
        }
        let mut not_have = Vec::new();
        for i in 0..lr.closeure.len() {
            if !used.contains(&i) {
                not_have.push(i);
            }
        }//将未使用的闭包编号放入

        let first = lr.lang.first_word.clone();
        let mut index = lr.index.len() + 1;
        for i in 0..lr.index.len() {
            if lr.index[i] == (first.clone(), 0) {
                index = i;
            }
        }
        let first_gen = lr.gen_list[index].clone();
        for c in 0..lr.closeure.len() {
            if lr.closeure[c].contains(&first_gen.2) {
                index = c;
                break;
            }
        }//获得起始闭包的编号

        let mut fact = vec![index];
        let mut changed = true;
        while changed {
            changed = false;
            for i in fact.clone() {
                let next = match way.get(&i) {
                    Some(v) => v,
                    None => continue,
                };
                for n in next {
                    if !fact.contains(n) {
                        fact.push(*n);
                        changed = true;
                    }
                }
            }
        }
        for i in 0..lr.closeure.len() {
            if !fact.contains(&i) {
                if !not_have.contains(&i){
                    not_have.push(i);//将不可及的闭包放入待删除列表
                }
            }
        }
        not_have.sort();
        not_have.reverse();
        for i in not_have {
            lr.closeure.remove(i);
        }//删除所有不可及和未使用的闭包


        let mut should_be_append: Vec<(usize, usize)> = Vec::new();
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..lr.closeure.len() {
                let c = lr.closeure[i].clone();
                for g in c {
                    let mut gen = g.clone();
                    let word = gen.get_current_place();
                    if gen.move_to_next() {
                        for s in 0..lr.closeure.len() {
                            if lr.closeure[s].contains(&gen) {
                                match go.go.insert((i, word.clone()), s) {
                                    Some(v) => {
                                        if (!should_be_append.contains(&(s, v))
                                            && !should_be_append.contains(&(v, s)))
                                            && s != v
                                        {
                                            if v > s {
                                                should_be_append.push((s, v));
                                            } else {
                                                should_be_append.push((v, s));
                                            }

                                            changed = true
                                        }
                                    } //当出现一个闭包使用相同的输入可以推出多个闭包的时候，标记合并闭包
                                    None => {}
                                }; //找到goto的方向，填入goto表(其中gen.get_current_place()可以获取移动前待识别的字符)
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
            }
            go.go.clear();
            let mut remove = Vec::new();
            for a in should_be_append.clone() {
                for c in lr.closeure[a.1].clone() {
                    if !lr.closeure[a.0].contains(&c) {
                        lr.closeure[a.0].push(c);
                    }
                }
                remove.push(a.1);
            }
            remove.sort();
            remove.reverse();
            for i in remove {
                lr.closeure.remove(i);
            }
            should_be_append.clear();
        }

        for i in 0..lr.closeure.len() {
            let c = lr.closeure[i].clone();
            for g in c {
                let mut gen = g.clone();
                let word = gen.get_current_place();
                if gen.move_to_next() {
                    for s in 0..lr.closeure.len() {
                        if lr.closeure[s].contains(&gen) {
                            go.go.insert((i, word.clone()), s); //找到goto的方向，填入goto表(其中gen.get_current_place()可以获取移动前待识别的字符)
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        go
    }
}

impl Display for GO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        for entity in self.go.clone() {
            write!(f, "c{} --{}-> c{}\n", (entity.0).0, (entity.0).1, entity.1)?;
        }
        write!(f, "")
    }
}

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
