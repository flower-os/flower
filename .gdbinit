target remote localhost:1234
symbol-file build/debug/kernel.bin
break kmain
python
def ignore_error(x): 
    try:
        gdb.execute(x)
    except:
        pass
ignore_error("continue")
end
disconnect
set architecture i386:x86-64
target remote localhost:1234
