use std::path::Path;

fn main() {
    let file_path = "/home/user/documents/file.txt";
    let parent_dir = Path::new(file_path).parent();

    match parent_dir {
        Some(dir) => println!("文件所在目录: {}", dir.display()),
        None => println!("无法获取目录"),
    }
}