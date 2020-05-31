from ctypes import cdll, c_char_p, c_void_p, Structure, POINTER
import os
import ctypes

class FFdummy(Structure):
    pass

lib = cdll.LoadLibrary("C:\\Users\\theny\\rust\\ll-1\\target\\release\\ll_1.dll")

lib.load_setting.restype = POINTER(FFdummy)

lib.free_setting.argtype = (POINTER(FFdummy),)

lib.analyze.argtype = (POINTER(FFdummy), c_char_p)
lib.analyze.restype = c_void_p#防止python将内存错误释放

lib.get_history.argtype = (POINTER(FFdummy), )
lib.get_history.restype = c_void_p

lib.free_str.argtype = (c_void_p, )

class FF:
    def __init__(self, conf_path, t_path, nt_path):
        self.obj = lib.load_setting(conf_path.encode("utf-8"), 
                        t_path.encode("utf-8"),
                        nt_path.encode("utf-8"))

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        lib.free_setting(self.obj) #释放内存

    def analyze(self, s):
        return lib.analyze(self.obj, s.encode("utf-8"))

    def get_history(self):
        return lib.get_history(self.obj)

def analyze(lang):
    with FF("language.txt", "NT.txt", "T.txt") as ff:
        ptr = ff.analyze(lang)
        history = ff.get_history()
        return ctypes.cast(ptr, c_char_p).value.decode("utf-8"), ctypes.cast(history, c_char_p).value.decode("utf-8")

lib.test()
res, his = analyze("i+i*(i+i)#")
print("result is :{} \nhistory is :".format(res))
print(his)