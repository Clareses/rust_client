use regex::Regex;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::{stdout, Write};
use std::io::{BufRead, BufReader};

fn format_print(content: &str, depth: usize, format: &str) {
    for _i in 0..depth {
        print!("  ");
    }

    if format == "file" {
        println!("Visit File : {}", content);
    } else if format == "enter_folder" {
        println!("Enter into Folder: {}", content);
    } else if format == "exit_folder" {
        println!("Exit From Folder:  {}", content);
    }
}

fn visit_folder(url: &str, folder_name: &str, client: &Client, depth: usize) {
    format_print(folder_name, depth, "enter_folder");

    // get the response and contents
    let content = client.get(url).send().unwrap().text().unwrap();

    // visit all folders
    let regex = Regex::new(" href=\"(.*folder.*lid=18730.{0,})\" title=\".*\n?\">?(.*)</a>");
    for cap in regex.unwrap().captures_iter(content.as_str()) {
        let mut path = String::from("http://met2.fzu.edu.cn/meol/common/script/");
        path.push_str(&cap[1]);
        visit_folder(path.as_str(), &cap[2], client, depth + 1);
    }

    // visit all files in this folder
    let regex =
        Regex::new(" href=\"(.{0,}preview.*lid=18730.*)\" target.* title=\".*\n?\">?(.*)</a>");
    for cap in regex.unwrap().captures_iter(content.as_str()) {
        format_print(&cap[2], depth + 1, "file");
        let mut path = String::from("http://met2.fzu.edu.cn/meol/common/script/");
        path.push_str(&cap[1]);
        client.get(path.as_str()).send().unwrap();
    }

    format_print(folder_name, depth, "exit_folder");
}

fn main() {
    let fd = File::open("./.config.txt");
    let file = match fd {
        Ok(f) => f,
        Err(_e) => {
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
            new_file
        }
    };
    let lines = BufReader::new(file).lines();

    let mut username: String = String::new();
    let mut password: String = String::new();
    let mut _i: usize = 0;

    for line in lines {
        if _i == 0 {
            username = line.unwrap();
        } else if _i == 1 {
            password = line.unwrap();
        }
        _i = _i + 1;
    }

    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let mut body: String = String::from("IPT_LOGINUSERNAME=");
    body.push_str(username.as_str());
    body.push_str("&IPT_LOGINPASSWORD=");
    body.push_str(password.as_str());

    client
        .post("http://met2.fzu.edu.cn/meol/loginCheck.do")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .unwrap();

    visit_folder(
        "http://met2.fzu.edu.cn/meol/common/script/listview.jsp?lid=18730&folderid=0",
        "Root",
        &client,
        0,
    );
}
