use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use crate::app::SerializableState;
use crate::util::default_repo_work_path;

pub fn get_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()),
        short_stamp
    )
}

pub fn get_previous_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    let mut num = short_stamp.parse::<i32>().unwrap();
    num = num - 1;
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()),
        num.to_string()
    )
}

pub fn save_state(app_state: SerializableState) {
    let path = get_state_path();

    let path_old = get_previous_state_path();

    let mut dir_only = path.to_owned();
    dir_only = dir_only.replace(path.split('/').last().unwrap(), "");

    if let Err(_e) = fs::create_dir_all(Path::new(dir_only.as_str())) {
        // println!("{:?}", e)
    }

    if let Err(_e) = fs::remove_file(Path::new(&path_old)) {
        // println!("{:?}", e)
    }
    // if let Err(_e) = fs::copy(&path, &path_old) {
    //     // println!("{:?}", e)
    // }
    if let Err(_e) = fs::remove_file(Path::new(&path)) {
        // println!("{:?}", e)
    }
    let state_string = serde_json::to_string(&app_state).unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(&path))
    {
        let _ = file.write(state_string.as_ref());
    }
    // println!("Saving state .. {}", path);
}
