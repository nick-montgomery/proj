use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct Tmux {
    label: String,
}

impl Tmux {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    fn base(&self) -> Command {
        let mut c = Command::new("tmux");
        c.arg("-L").arg(&self.label); // dedicate a server to avoid collisions
        c
    }

    pub fn run<'a>(&self, args: impl IntoIterator<Item = &'a str>) -> Result<()> {
        let status = self.base().args(args).status()?;
        if !status.success() {
            anyhow::bail!("tmux exited with code {:?}", status.code());
        }
        Ok(())
    }

    pub fn out<'a>(&self, args: impl IntoIterator<Item = &'a str>) -> Result<String> {
        let out = self.base().args(args).output()?;
        if !out.status.success() {
            anyhow::bail!(
                "tmux command failed (code {:?}\nstout:\n{}\nstderr\n{}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr),
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    pub fn ok<'a>(&self, args: impl IntoIterator<Item = &'a str>) -> Result<bool> {
        Ok(self
            .base()
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success())
    }
}

pub fn ensure_server(tmux: &Tmux) -> Result<()> {
    tmux.run(["start-server"]).context("starting tmux server")
}

pub fn ensure_windows(tmux: &Tmux, session: &str, name: &str, cwd: &Path) -> Result<()> {
    if !window_exists(tmux, session, name)? {
        tmux.run([
            "new-window",
            "-t",
            session,
            "-n",
            name,
            "-c",
            cwd.to_str().unwrap(),
        ])?;
    }
    Ok(())
}

pub fn window_exists(tmux: &Tmux, session: &str, name: &str) -> Result<bool> {
    let out = tmux.out(["list-windows", "-t", session, "-F", "#{window_name}"])?;
    Ok(out.lines().any(|w| w == name))
}

pub fn setup_docker_layout(tmux: &Tmux, session: &str) -> Result<Vec<String>> {
    let target = format!("{session}:docker");
    let _ = tmux.run(["select-window", "-t", &target]);
    tmux.run(["kill-pane", "-a", "-t", &target])?;
    tmux.run(["select-layout", "-t", &target, "tiled"])?;
    tmux.run(["split-window", "-h", "-t", &target])?;
    tmux.run(["split-window", "-v", "-t", &format!("{target}.0")])?;
    tmux.run(["split-window", "-v", "-t", &format!("{target}.1")])?;
    let out = tmux.out(["list-panes", "-t", &target, "-F", "#{pane_index:#{pane_id}"])?;
    let mut panes = vec!["".to_string(); 4];
    for line in out.lines() {
        if line
            .split_once(':')
            .and_then(|(i, id)| i.parse::<usize>().ok().map(|idx| (idx, id)))
            .is_some_and(|(idx, id)| {
                if idx < panes.len() {
                    panes[idx] = id.into();
                    true
                } else {
                    false
                }
            })
        {}
    }
    Ok(panes)
}

pub fn send_to_target_sh(tmux: &Tmux, target: &str, cwd: &Path, cmd: &str) -> Result<()> {
    let cd = shell_escape(cwd);
    let line = format!("cd {cd} && clear && {cmd}");
    tmux.run(["send-keys", "-t", target, &line, "C-m"])
}

pub fn attach_or_switch(tmux: &Tmux, session: &str) -> Result<()> {
    if env::var_os("TMUX").is_some() {
        tmux.run(["switch-client", "-t", session])
    } else {
        tmux.run(["attach-session", "-t", session])
    }
}

fn shell_escape(p: &Path) -> String {
    let s = p.to_string_lossy();
    format!("'{}'", s.replace('\'', r"'\''"))
}
