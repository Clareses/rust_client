use regex::Regex;
use reqwest::blocking::Client;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{stdout, Write};
use std::io::{BufRead, BufReader};

// init the config file
fn init_config_file() -> File {
    let mut username = String::new();
    let mut password = String::new();
    let mut new_file = File::create("./.config.txt").unwrap();
    print!("Input username:");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut username).unwrap();
    print!("Input password:");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut password).unwrap();
    write!(new_file, "{}", username).unwrap();
    write!(new_file, "{}", password).unwrap();
    new_file.flush().unwrap();
    File::open("./.config.txt").unwrap()
}

// read config file
fn read_config_file(file: &mut File) -> (String, String) {
    let mut username: String = String::new();
    let mut password: String = String::new();
    let mut reader = BufReader::new(file);
    reader.read_line(&mut username).unwrap();
    reader.read_line(&mut password).unwrap();
    username.pop();
    password.pop();
    (username, password)
}

// read record file
fn read_record_file(file: &File) -> BTreeSet<String> {
    let mut tree = BTreeSet::new();
    let lines = BufReader::new(file).lines();
    for line in lines {
        tree.insert(line.unwrap());
    }
    tree
}

fn update_recorder(mut file: File, tree: BTreeSet<String>) {
    for node in tree.into_iter() {
        write!(file, "{}\n", node).unwrap();
    }
    file.flush().unwrap();
}

fn format_print(content: &str, depth: usize, format: &str) {
    for _i in 0..depth {
        print!("  ");
    }

    if format == "new_file" {
        println!("\x1b[1;42;30mFIND NEW FILE : {}\x1b[40;37m", content);
    } else if format == "enter_folder" {
        println!("Enter into Folder: {}", content);
    } else if format == "exit_folder" {
        println!("Exit From Folder:  {}", content);
    } else if format == "old_file" {
        println!("old file : {}", content);
    }
}

fn visit_folder(
    url: &str,
    folder_name: &str,
    client: &Client,
    depth: usize,
    recorder: &mut BTreeSet<String>,
) {
    format_print(folder_name, depth, "enter_folder");

    // get the response and contents
    let content = client.get(url).send().unwrap().text().unwrap();

    // visit all folders
    let regex = Regex::new(" href=\"(.*folder.*lid=18730.{0,})\" title=\".*\n?\">?(.*)</a>");
    for cap in regex.unwrap().captures_iter(content.as_str()) {
        let mut path = String::from("http://met2.fzu.edu.cn/meol/common/script/");
        path.push_str(&cap[1]);
        visit_folder(path.as_str(), &cap[2], client, depth + 1, recorder);
    }

    // visit all files in this folder
    let regex =
        Regex::new(" href=\"(.*preview.*lid=18730.*)\".*target.*\n?.*title=\".*\n?\">?(.*)</a>");

    for cap in regex.unwrap().captures_iter(content.as_str()) {
        match recorder.contains(&cap[1]) {
            true => format_print(&cap[2], depth + 1, "old_file"),
            false => {
                format_print(&cap[2], depth + 1, "new_file");
                let mut path = String::from("http://met2.fzu.edu.cn/meol/common/script/");
                path.push_str(&cap[1]);
                client.get(path.as_str()).send().unwrap();
                recorder.insert(String::from(&cap[1]));
            }
        }
    }

    format_print(folder_name, depth, "exit_folder");
}

fn main() {
    // open config file and record file
    let fd = File::open("./.config.txt");
    let mut config = match fd {
        Ok(f) => f,
        Err(_e) => init_config_file(),
    };

    let fd = File::open("./.recorder.txt");
    let recorder = match fd {
        Ok(f) => f,
        Err(_e) => {
            File::create("./.recorder.txt").unwrap();
            File::open("./.recorder.txt").unwrap()
        }
    };

    // read the config file
    let (username, password) = read_config_file(&mut config);

    //read the recod file and make a tree
    let mut tree = read_record_file(&recorder);

    // init client
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // prepare request body
    let mut body: String = String::from("IPT_LOGINUSERNAME=");
    body.push_str(username.as_str());
    body.push_str("&IPT_LOGINPASSWORD=");
    body.push_str(password.as_str());

    // login and get the cookie
    client
        .post("http://met2.fzu.edu.cn/meol/loginCheck.do")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .unwrap();

    // visit the root folder
    visit_folder(
        "http://met2.fzu.edu.cn/meol/common/script/listview.jsp?lid=18730&folderid=0",
        "Root",
        &client,
        0,
        &mut tree,
    );

    client
        .get("http://met2.fzu.edu.cn/meol/homepage/common/logout.jsp")
        .send()
        .unwrap();

    let recorder = File::create("./.recorder.txt").unwrap();
    update_recorder(recorder, tree);
}
