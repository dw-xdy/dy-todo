use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use rodio::Decoder;
use walkdir::WalkDir;

// 结构体表示音频文件
#[derive(Debug)]
struct AudioFile {
    path: PathBuf,
    name: String,
}

// 扫描目录获取所有MP3文件
fn scan_mp3_files(directory: &str) -> Result<Vec<AudioFile>, Box<dyn std::error::Error>> {
    let mut mp3_files = Vec::new();

    for entry in WalkDir::new(directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file()
            && let Some(ext) = path.extension()
            && (ext.to_ascii_lowercase() == "mp3" || ext.to_ascii_lowercase() == "wav")
        {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            mp3_files.push(AudioFile {
                path: path.to_path_buf(),
                name,
            });
        }
    }

    mp3_files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(mp3_files)
}

fn play_mp3_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

    // 2. 创建 Sink。
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    // 3. 打开并解码
    let file = File::open(path)?;
    let source = Decoder::new(BufReader::new(file))?;

    println!("正在播放: {}", path.display());

    sink.append(source);

    // 4. 阻塞当前线程直到播放结束
    sink.sleep_until_end();

    Ok(())
}

// 批量播放MP3文件
fn play_mp3_files(files: &[AudioFile]) -> Result<(), Box<dyn std::error::Error>> {
    for (i, audio_file) in files.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, files.len(), audio_file.name);

        match play_mp3_file(&audio_file.path) {
            Ok(_) => println!("播放完成: {}", audio_file.name),
            Err(e) => eprintln!("播放失败 {}: {}", audio_file.name, e),
        }

        // 如果不是最后一个文件，添加短暂暂停
        if i < files.len() - 1 {
            println!("准备下一曲...");
            thread::sleep(Duration::from_secs(2));
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置要扫描的目录（可以修改为你的目录）
    let target_directory = "F:\\D\\音乐\\音乐文件"; // 修改为你的目录路径

    if !Path::new(target_directory).exists() {
        eprintln!("目录不存在: {target_directory}");
        return Ok(());
    }

    println!("正在扫描目录: {target_directory}");

    // 扫描MP3文件
    let mp3_files = scan_mp3_files(target_directory)?;

    if mp3_files.is_empty() {
        println!("在目录 {target_directory} 中未找到MP3 / WAV文件");
        return Ok(());
    }

    println!("找到 {} 个MP3文件:", mp3_files.len());
    for (i, file) in mp3_files.iter().enumerate() {
        println!("  {}. {}", i + 1, file.name);
    }

    // 播放所有文件
    play_mp3_files(&mp3_files)?;

    println!("所有曲目播放完毕！");
    Ok(())
}
