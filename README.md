# LUIGI-FLOWEY

![image](https://github.com/user-attachments/assets/1fd22c83-c82c-415a-926e-cfe28697a8b5)


![alt text](image.png)

## I'm getting some "vcredist140.dll" error

usually, this error doesn't appear if you've
somehow installed a bunch of stuff on your computer

idk how to solve this otherwise.

## How to compile

Alongside your regular "rust" compilation rules:

In order for this to work:

- Have VS 2022 installed and make sure you install the basics for the "2022" one
- Reroute the CMake path to the Visual Studio 2022 one (in `C:/Program Files/Visual Studio 2022/`) search `cmake.exe`, then add the folder where CMake is into your system path above most things else
- Have LLVM installed, `winget install LLVM.LLVM`
- Set `LIBCLANG_PATH`, the environment variable, to the LLVM bin folder (probably in program files / LLVM)
