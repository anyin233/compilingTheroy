mod ff;
mod language;
mod history;

fn main() {
    let mut lg = language::Language::start();
    lg.new("language.txt", "NT.txt", "T.txt");
    let mut ff = ff::FF::new(lg);
    println!("{}", ff);
    println!("{}", ff.analyze("i+i*(i+i)#".to_owned()));
}
