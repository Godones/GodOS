use std::fs::{read_dir, File};
use std::io::{Result, Write};

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

///生成三个应用二进制数据的相关信息
fn main() {
    println!("cargo:rerun-if-changed=../user/src");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    build_app_data().unwrap();
}

fn build_app_data() -> Result<()> {
    let mut link_app = File::create("src/link_app.S")?;
    let mut apps: Vec<_> = read_dir("../user/src/bin")?
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();
    writeln!(
        link_app,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}
    "#,
        apps.len()
    )?;
    for i in 0..apps.len() {
        writeln!(link_app, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(link_app, r#"  .quad app_{}_end"#, apps.len() - 1)?;
    writeln!(
        link_app,
        r#"
    .global _app_name
_app_name:"#
    )?;
    for app in apps.iter() {
        writeln!(link_app, r#"    .string "{}""#, app)?;
    }

    for (index, app) in apps.iter().enumerate() {
        writeln!(
            link_app,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            index, app, TARGET_PATH
        )?;
    }

    Ok(())
}
