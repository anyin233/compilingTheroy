mod ff;
mod language;


fn main() {
    let mut lg = language::Language::start();
    lg.new("language.txt", "NT.txt", "T.txt");
    let ff = ff::FF::new(lg);
    println!("{}", ff);
    println!("{}", ff.analyze("i+i#".to_owned()));
}
