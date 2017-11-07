# Boot Error Code List

| Hex Code | Description     | Cause                                                          | Note                                  |
|----------|-----------------|----------------------------------------------------------------|---------------------------------------|
| `0x1`    | Multiboot error | Multiboot didn't boot correctly; `eax` not set to magic number | Error with GRUB?                      |
| `0x2`    | No `cpuid`      | `cpuid` isn't supported on the cpu.                            | This could be due to using an old CPU |
| `0x3`    | No long mode    | Long mode isn't supported by the cpu                           | The CPU most likely isn't 64 bit      |
