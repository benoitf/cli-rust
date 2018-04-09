extern crate clap;

use std::io;
use std::process;
use std::process::{Command, Stdio};
use std::io::Write;
use clap::{App, Arg};

fn main() {
    let matches = App::new("Eclipse Che CLI")
        .version("0.0.1")
        .author("Florent Benoit <fbenoit@redhat.com>")
        .about("Native Eclipse Che CLI")
        .arg(
            Arg::with_name("command")
                .required(true)
                .takes_value(true)
                .possible_values(&["deploy"])
                .index(1)
                .help("command to execute"),
        )
        .arg(Arg::with_name("option").multiple(true))
        .get_matches();

    /*let option_values = matches.values_of("option");
    if !option_values.is_none() {
        let unwrap = option_values.unwrap();
        //println!("raw options: {:?}", unwrap.collect::<Vec<_>>().join(" "));
    }*/

    let _command = matches.value_of("command").unwrap();
    //println!("executing command with name {}", command);

    // display logo
    let logo_bytes = include_bytes!("logo.txt");
    println!("{}", String::from_utf8_lossy(logo_bytes).to_string());


    let oc_command_check = Command::new("oc").arg("help").output();
    match oc_command_check {
        Err(e) => {
            println!("The oc tool has not been found on the path. This tool is required to setup Eclipse Che on OpenShift. {}", e);
            process::exit(1);
        }
        Ok(_v) => {}
    }

    let mut create_project = true;
    let mut project_name = String::from("eclipse-che");
    let mut http_support = false;
    println!("OpenShift configuration:");
    println!("-> Create a new project ? [Y/n]");

    let mut line = String::new();

    while let Ok(_n) = io::stdin().read_line(&mut line) {
        let choice = line.trim().to_ascii_lowercase();

        match choice.as_ref() {
            "" | "y" => {
                create_project = true;
                break;
            }
            "n" => {
                create_project = false;
                break;
            }
            _ => {
                println!("Invalid answer");
            }
        }
    }
    if create_project {
        println!("-> Project Name ? [{}]", project_name);

        while let Ok(_n) = io::stdin().read_line(&mut line) {
            let choice = line.trim().to_ascii_lowercase();

            match choice.as_ref() {
                "" => {
                    println!("Using default name {}", project_name);
                    break;
                }
                _ => {
                    project_name = choice;
                    break;
                }
            }
        }
    }

    println!("-> project name = {}", project_name);
    println!("-> Https support ? [y/N]");
    while let Ok(_n) = io::stdin().read_line(&mut line) {
        let choice = line.trim().to_ascii_lowercase();

        match choice.as_ref() {
            "y" => {
                http_support = true;
                break;
            }
            "" | "n" => {
                http_support = false;
                break;
            }
            _ => {
                println!("Invalid answer");
            }
        }
    }
    println!("Configuration:");
    println!(" HTTPS: {}", http_support);

    // create project ?
    if create_project {
        let create_project_command = Command::new("oc")
        .arg("new-project")
        .arg(project_name)
        .stdin(Stdio::piped()) // use custom
        .stdout(Stdio::piped()) // save stdout
        .stderr(Stdio::piped()) // save stdout
        .spawn()
        .unwrap();

        let output = create_project_command.wait_with_output().unwrap();
        let stdout_create_project = String::from_utf8(output.stdout).unwrap();
        let stderr_create_project = String::from_utf8(output.stderr).unwrap();

        if output.status.success() {
            println!("Project created successfully");
        } else {
            println!("Unable to create new project");
            println!("{}", stdout_create_project);
            println!("{}", stderr_create_project);
            process::exit(1);
        }
    }

    let template_bytes = include_bytes!("che-server-template.yaml");
    let routing_suffix = format!("ROUTING_SUFFIX={}", "192.168.64.7.nip.io");

    let mut cmd_grep = Command::new("oc")
        .arg("new-app")
        .arg("-p")
        .arg(routing_suffix)
        .arg("-f")
        .arg("-")
        .stdin(Stdio::piped()) // use custom
        .stdout(Stdio::piped()) // save stdout
        .stderr(Stdio::piped()) // save stdout
        .spawn()
        .unwrap();

    // write template to stdin
    if let Some(ref mut stdin) = cmd_grep.stdin {
        stdin.write_all(template_bytes).unwrap();
    }

    let output = cmd_grep.wait_with_output().unwrap();
    let s = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    if output.status.success() {
        let oc_command_route = Command::new("oc").arg("get").arg("route").arg("che").arg("-o").arg("jsonpath={.spec.host}").output();
    match oc_command_route {
        Err(e) => {
            println!("Unable to get the route. {}", e);
            process::exit(1);
        }
        Ok(v) => {
            println!("Che successfully deployed: Connect to http://{}", String::from_utf8(v.stdout).unwrap());
        }
    }

    } else {
        println!(
            "Unable to create new Eclipse Che app {:?}",
            output.status.code().unwrap()
        );
        println!("stdout is {}", s);
        println!("stderr is {}", stderr);
        process::exit(1);
    }



}
