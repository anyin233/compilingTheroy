from ctypes import cdll, c_char_p, Structure, POINTER
import os

class FFdummy(Structure):
    pass

lib = cdll.LoadLibrary("C:\\Users\\theny\\rust\\ll-1\\target\\debug\\ll_1.dll")
lib.test()

s = "Hello".encode("utf-8")
lib.say_hello(s)

