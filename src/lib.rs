extern crate regex;
extern crate lazy_static;

mod ff;
mod language;
mod history;
mod lr;

use libc::{c_char};
use std::ffi::{CStr, CString};
use language::Language;
use ff::FF;
use std::boxed::Box;
use lr::LR;

#[no_mangle]
extern fn test() {
    let mut lg = Language::start();
    lg.new("language.txt", "NT.txt", "T.txt", false);
    let mut ff = FF::new(&lg);
    println!("{}", ff);
    println!("{}", ff.analyze("i+i*(i+i)#".to_owned()));
}

#[no_mangle]
extern fn say_hello(ptr: *const c_char){
    let cstr = unsafe{
        assert!(!ptr.is_null());

        CStr::from_ptr(ptr)
    };
    let rstr = cstr.to_str().unwrap();
    println!("{}", rstr);
}

#[no_mangle]
extern fn load_setting(conf_path: *const c_char, t_path: *const c_char, nt_path: *const c_char) -> *mut FF{//创建语言解析器
    let conf = convert_c_char(conf_path);
    let t = convert_c_char(t_path);
    let nt = convert_c_char(nt_path);

    let mut lg = Language::start();
    lg.new(conf.as_str(), t.as_str(), nt.as_str(), false);
    let ff = FF::new(&lg);
    Box::into_raw(Box::new(ff))
}//创建FF1解释器

#[no_mangle]
extern fn load_from_string(conf: *const c_char, t: *const c_char, nt: *const c_char) -> *mut FF{
    let conf = convert_c_char(conf);
    let t = convert_c_char(t);
    let nt = convert_c_char(nt);

    let mut lg = Language::start();
    lg.new_from_string(conf, nt, t, false);
    let ff = FF::new(&lg);
    Box::into_raw(Box::new(ff))
}

#[no_mangle]
extern fn free_setting(ptr:*mut FF){
    if ptr.is_null(){
        return;
    }
    unsafe{Box::from_raw(ptr);}
}//从内存中释放FF1解释器

#[no_mangle]
extern fn analyze(ptr: *mut FF, s:*const c_char) -> *mut c_char{
    let ff = unsafe{
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let st = convert_c_char(s);
    let res = ff.analyze(st);
    let res_c = CString::new(res).unwrap();
    res_c.into_raw()
}//使用创建的FF对现有的语句进行分析，并返回分析结果(String)

#[no_mangle]
extern fn get_history(ptr: *const FF) -> *mut c_char{
    let ff = unsafe{
        assert!(!ptr.is_null());
        &*ptr
    };
    let his = ff.get_history();
    let his_c = CString::new(his).unwrap();
    his_c.into_raw()
}//返回分析历史


#[no_mangle]
extern fn free_str(s:*mut c_char){
    unsafe{
        if s.is_null(){return};
        CString::from_raw(s);
    }
}//由rust释放字符串内存


///========================
/// LR分析器
/// load_from_string_lr() => 加载lr分析器(使用string)
/// analyze_lr() => 使用分析器进行分析
/// get_history_lr() => 获取lr分析过程的历史

#[no_mangle]
extern fn load_setting_lr(conf_path: *const c_char, t_path: *const c_char, nt_path: *const c_char) -> *mut LR{//创建语言解析器
    let conf = convert_c_char(conf_path);
    let t = convert_c_char(t_path);
    let nt = convert_c_char(nt_path);

    let mut lg = Language::start();
    lg.new(conf.as_str(), t.as_str(), nt.as_str(), true);
    let ff = LR::new(&lg);
    Box::into_raw(Box::new(ff))
}//创建FF1解释器

#[no_mangle]
extern fn load_from_string_lr(conf: *const c_char, t: *const c_char, nt: *const c_char) -> *mut LR{
    let conf = convert_c_char(conf);
    let t = convert_c_char(t);
    let nt = convert_c_char(nt);

    let mut lg = Language::start();
    lg.new_from_string(conf, nt, t, true);
    let lr = LR::new(&lg);
    Box::into_raw(Box::new(lr))
}

#[no_mangle]
extern fn analyze_lr(ptr: *mut LR, s:*const c_char) -> *mut c_char{
    let ff = unsafe{
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let st = convert_c_char(s);
    let res = ff.analyze(st);
    let res_c = CString::new(res).unwrap();
    res_c.into_raw()
}//使用创建的LR对现有的语句进行分析，并返回分析结果(String)

#[no_mangle]
extern fn get_history_lr(ptr: *const LR) -> *mut c_char{
    let ff = unsafe{
        assert!(!ptr.is_null());
        &*ptr
    };
    let his = ff.get_history();
    let his_c = CString::new(his).unwrap();
    his_c.into_raw()
}//返回分析历史

#[no_mangle]
extern fn free_setting_lr(ptr:*mut LR){
    if ptr.is_null(){
        return;
    }
    unsafe{Box::from_raw(ptr);}
}//从内存中释放FF1解释器

fn convert_c_char(s: *const c_char) -> String{
    let s = unsafe{
        assert!(!s.is_null());

        CStr::from_ptr(s)
    };
    let s_str = match s.to_str() {
        Ok(v) => v.to_owned(),
        Err(e) => {
            println!("Cannot Convert, Error is {}", e);
            "".to_owned()
        }
    };
    s_str
}

#[test]
fn test_builder(){
    
    let conf_path = "language.txt";
    let t_path = "T.txt";
    let nt_path = "NT.txt";
    let conf = CString::new(conf_path).unwrap().into_raw();
    let t = CString::new(t_path).unwrap().into_raw();
    let nt = CString::new(nt_path).unwrap().into_raw();
    let ff = load_setting(conf, nt, t);
    assert!(!ff.is_null());
    let lg = "i+i*(i+i)#";
    let l = CString::new(lg).unwrap().into_raw();
    let s = analyze(ff, l);
    let st = convert_c_char(s);
    assert!(st.contains("Succeed"))
}

#[test]
fn lr_tester(){
    let conf_path = "language.txt";
    let t_path = "T.txt";
    let nt_path = "NT.txt";
    let conf = CString::new(conf_path).unwrap().into_raw();
    let t = CString::new(t_path).unwrap().into_raw();
    let nt = CString::new(nt_path).unwrap().into_raw();
    let ff = load_setting_lr(conf, nt, t);
    assert!(!ff.is_null());
    let lg = "aabab";
    let l = CString::new(lg).unwrap().into_raw();
    let s = analyze_lr(ff, l);
    let st = convert_c_char(s);
    let his= get_history_lr(ff);
    let h = convert_c_char(his);
    println!("{}", h);
    assert!(st.contains("Succeed"))
}

#[test]
fn lr(){
    let conf_path = "language.txt";
    let t_path = "T.txt";
    let nt_path = "NT.txt";
    let mut lang = Language::start();
    lang.new(conf_path, t_path, nt_path, true);
    let lr = lr::LR::new(&lang);
    println!("{}", lr);
}

#[test]
fn test_left_re(){
    let conf_path = "language.txt";
    let t_path = "T.txt";
    let nt_path = "NT.txt";
    let mut lang = Language::start();
    lang.new(conf_path, t_path, nt_path, true);
    let mut lr = lr::LR::new(&lang);
    println!("\n{}", lr);
    println!("{}", lr.analyze("()".to_owned()));
    println!("{}", lr.get_history());
}