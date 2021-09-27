#### OS_base

##### 目标平台和三元组

对于程序源代码而言，编译器在将其通过编译、链接得到可执行文件的时候需要知道程序要在哪个 **平台** (Platform) 上运行。这里 **平台** 主要是指CPU类型、操作系统类型和标准运行时库的组合。

 **目标三元组** (Target Triplet) 用来描述一个目标平台。

```
host: x86_64-unknown-linux-gnu
```

`cpu架构`：x86-64

`cpu厂商`: unknow

`操作系统`: linux

`运行时库`: gnu libc

**Rust 编译器支持下面的基于RISC v的平台**

```
riscv32gc-unknown-linux-gnu
riscv32gc-unknown-linux-musl
riscv32i-unknown-none-elf
riscv32imac-unknown-none-elf
riscv32imc-esp-espidf
riscv32imc-unknown-none-elf
riscv64gc-unknown-linux-gnu
riscv64gc-unknown-linux-musl
riscv64gc-unknown-none-elf
riscv64imac-unknown-none-elf
```

<font color ='red '>选择riscv64gc-unknown-none-elf</font>

没有操作系统支持，也没有如何运行时库，可以生成elf格式的文件。

##### 移除标准库支持：

当我们在写出下述代码时：

```rust
fn main(){
    println!("hello world");
}
```

println! 宏所在的Rust标准库std需要通过系统调用获得操作系统的服务。但此时我们没有操作系统没有系统调用，因此需要去掉这个宏。

```rust
#![no_std]
fn main(){}
```

- Rust 的标准库–std，为绝大多数的 Rust 应用程序开发提供基础支持、跨硬件和操作系统平台支持，是应用范围最广、地位最重要的库，但需要有底层操作系统的支持。
- Rust 的核心库–core，可以理解为是经过大幅精简的标准库，它被应用在标准库不能覆盖到的某些特定领域，如裸机(bare metal) 环境下，用于操作系统和嵌入式系统的开发，它不需要底层操作系统的支持

**我们不能使用Rust的标准库，但可以使用核心库，核心库不依赖于操作系统**

移除掉标准库后，此时再使用`cargo run`运行会产生下列错误

```
error: `#[panic_handler]` function required, but not found
```

核心库中不存在这个语义项的实现，因此我们需要实现它

```rust

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info:&PanicInfo)->!{
    loop{}
}
```

加入此语义项再允许`cargo run`会产生下列错误

```
error: requires `start` lang_item
```

语言标准库和三方库作为应用程序的执行环境，需要负责在执行应用程序之前进行一些初始化工作，然后才跳转到应用程序的入口点（也就是跳转到我们编写的 `main` 函数）开始执行。事实上 `start` 语义项代表了标准库 std 在执行应用程序之前需要进行的一些初始化工作。由于我们禁用了标准库，编译器也就找不到这项功能的实现了。

最粗暴的方式我们可以直接删除`main`函数并告诉编译器没有main函数即可。



```txt
#注意，在不同目标平台上，cargo build产生的错误可能不一致
#在x86_64-unknown-linux-gnu平台上，会报下面的错误
error: language item required, but not found: `eh_personality`
```

`eh_personality` 标记一个函数用于实现 `stack unwinding` 。默认情况下，当出现 `panic` 时，Rust使用unwinding调用所有stack上活动变量的destructors，以完成内容的释放，确保父线程catch panic异常并继续执行。Unwinding是个复杂的操作，并且依赖一些OS库支持，因为我们正在编写OS，因此这里不能使用Unwinding

关闭Unwinding的简单方法即在`cargo.toml`文件设置

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

上述设计起到的作用即时在panic时直接退出。

