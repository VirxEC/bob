use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::{fs, time::Instant};

use crate::{
    buildinfo::{BuildInfo, Project},
    PathCommand,
};
use anyhow::anyhow;
use liblzma::read::XzEncoder;
use tar::Builder;

pub fn pack(command: PathCommand) -> anyhow::Result<()> {
    if !fs::exists(&command.build_dir)? {
        return Err(anyhow!("Directory doesn't exist"));
    }

    let buildinfo_path = command.build_dir.join("buildinfo.toml");
    if !fs::exists(&buildinfo_path)? {
        return Err(anyhow!("buildinfo.toml doesn't exist"));
    }

    let str_contents = fs::read_to_string(&buildinfo_path)?;
    let mut buildinfo: BuildInfo = toml::from_str(&str_contents)?;

    create_archives(&buildinfo.projects, &command.build_dir, "full")?;

    if let Some(old_buildinfo) = &command.old_buildinfo {
        if !fs::exists(old_buildinfo)? {
            eprintln!("Specified file for old buildinfo doesn't exist");
            // on first build, this file won't exist
            return Ok(());
        }

        let str_contents = fs::read_to_string(old_buildinfo)?;
        let old_buildinfo: BuildInfo = toml::from_str(&str_contents)?;

        // remove projects that haven't changed
        for old_project in old_buildinfo.projects {
            let project_idx = buildinfo
                .projects
                .iter()
                .position(|p| p.name == old_project.name);
            if let Some(idx) = project_idx {
                if buildinfo.projects[idx].hash == old_project.hash {
                    buildinfo.projects.remove(idx);
                }
            }
        }

        create_archives(&buildinfo.projects, &command.build_dir, "diff")?;
    }

    Ok(())
}

fn create_archives(projects: &[Project], build_dir: &Path, name: &str) -> anyhow::Result<()> {
    let compression_start = Instant::now();

    let platforms = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"];

    for platform in platforms {
        let archive_name = format!("{platform}-{name}.tar.xz");
        println!("Now creating archive {archive_name}");

        let mut a = Builder::new(Vec::new());

        let mut buildinfo = File::open(build_dir.join("buildinfo.toml"))?;
        a.append_file("buildinfo.toml", &mut buildinfo).unwrap();

        for project in projects {
            let project_dir = build_dir.join(&project.name);
            if !project_dir.exists() {
                return Err(anyhow!(
                    "Project directory for {} no longer exists",
                    project.name
                ));
            }

            let platform_dir = project_dir.join(platform);
            if !platform_dir.exists() {
                continue;
            }

            a.append_dir_all(format!("{}/{platform}", project.name), platform_dir)
                .unwrap();

            for entry in fs::read_dir(&project_dir)?.into_iter().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }

                let archive_file_name = format!(
                    "{}/{}",
                    project.name,
                    path.file_name().unwrap().to_str().unwrap()
                );

                let mut file = File::open(&path)?;
                a.append_file(archive_file_name, &mut file).unwrap();
            }
        }

        let raw_data = a.into_inner().unwrap();
        let mut compressed_data = XzEncoder::new(raw_data.as_slice(), 9);

        let mut file = File::create(build_dir.join(archive_name))?;
        copy(&mut compressed_data, &mut file)?;
    }

    println!(
        "Created {name} archives in {:2}s",
        compression_start.elapsed().as_secs_f32()
    );

    Ok(())
}
