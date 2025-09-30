use crate::tmux::{
    Tmux, attach_or_switch, ensure_server, ensure_windows, send_to_target_sh, setup_docker_layout,
};
use crate::utils::{compose_file, detect_dev_cmd, guess_backend_dir, guess_frontend_dir};
use anyhow::Result;
use std::path::Path;

const TMUX_LABEL: &str = "projctl"; // isolate from user's default tmux

pub fn setup_servers(proj_dir: &Path, refresh: bool, reset: bool, kill: bool) -> Result<()> {
    let proj_name = proj_dir
        .file_name()
        .expect("project dir has a name")
        .to_string_lossy()
        .to_string();
    let session = format!("{proj_name}-servers");

    let tmux = Tmux::new(TMUX_LABEL);

    ensure_server(&tmux)?;

    let mut has_session = tmux.ok(["has-session", "-t", &session])?;

    if kill {
        if has_session {
            println!("Killing session '{session}'...");
            tmux.run(["kill-session", "-t", &session])?;
        } else {
            println!("No session '{session}' to kill.");
        }
        return Ok(());
    }

    if reset && has_session {
        println!("Resetting session '{session}'...");
        tmux.run(["kill-session", "-t", &session])?;
        has_session = false;
    }

    if !has_session {
        println!("Creating session '{session}'...");
        tmux.run([
            "new-session",
            "-d",
            "-s",
            &session,
            "-n",
            "frontend",
            "-c",
            proj_dir.to_str().unwrap(),
        ])?;

        ensure_windows(&tmux, &session, "backend", proj_dir)?;
        ensure_windows(&tmux, &session, "docker", proj_dir)?;
        ensure_windows(&tmux, &session, "logs", proj_dir)?;
        ensure_windows(&tmux, &session, "scratch", proj_dir)?;

        let panes = setup_docker_layout(&tmux, &session)?;
        seed_frontend(&tmux, &session, proj_dir)?;
        seed_backend(&tmux, &session, proj_dir)?;
        seed_docker(&tmux, proj_dir, &panes)?;

        attach_or_switch(&tmux, &session)?;
        return Ok(());
    }

    if refresh {
        println!("Refreshing session '{session}' (reseed layout + commands).");

        ensure_windows(&tmux, &session, "frontend", proj_dir)?;
        ensure_windows(&tmux, &session, "backend", proj_dir)?;
        ensure_windows(&tmux, &session, "docker", proj_dir)?;
        ensure_windows(&tmux, &session, "logs", proj_dir)?;
        ensure_windows(&tmux, &session, "scratch", proj_dir)?;

        let panes = setup_docker_layout(&tmux, &session)?;
        seed_frontend(&tmux, &session, proj_dir)?;
        seed_backend(&tmux, &session, proj_dir)?;
        seed_docker(&tmux, proj_dir, &panes)?;

        attach_or_switch(&tmux, &session)?;
        return Ok(());
    }

    println!("Session '{session}' exists - attaching.");
    attach_or_switch(&tmux, &session)?;
    Ok(())
}

/* ------------------------- seeding -------------------- */

fn seed_frontend(tmux: &Tmux, session: &str, proj_dir: &Path) -> Result<()> {
    let front_dir = guess_frontend_dir(proj_dir).unwrap_or_else(|| proj_dir.to_path_buf());
    let front_cmd = detect_dev_cmd(&front_dir);
    send_to_target_sh(tmux, &format!("{session}:frontend"), &front_dir, &front_cmd)
}

fn seed_backend(tmux: &Tmux, session: &str, proj_dir: &Path) -> Result<()> {
    if let Some(bd) = guess_backend_dir(proj_dir) {
        let back_cmd = detect_dev_cmd(&bd);
        send_to_target_sh(tmux, &format!("{session}:backend"), &bd, &back_cmd)
    } else {
        send_to_target_sh(
            tmux,
            &format! {"{session}:backend"},
            proj_dir,
            "echo 'No backend dir found'; exec $SHELL",
        )
    }
}

fn seed_docker(tmux: &Tmux, proj_dir: &Path, panes: &[String]) -> Result<()> {
    // panes[0] => compose/watch
    if let Some(compose) = compose_file(proj_dir) {
        let path = compose.display().to_string();
        let cmd =
            format!("docker compose -f {path} up -d && watch -n 1 'docker compose -f {path} ps'");
        send_to_target_sh(tmux, &panes[0], proj_dir, &cmd)?;
    } else {
        let cmd = r#"watch -n 1 "docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}'""#;
        send_to_target_sh(tmux, &panes[0], proj_dir, cmd)?;
    }

    // panes[1] => Postgres logs (best-effort)
    let pg_cmd = r#"docker ps --format '{{.Names}} | grep -Ei 'postgres|pg' | head -n1 | xargs -r docker logs -f || echo 'No postgres'"#;
    send_to_target_sh(tmux, &panes[1], proj_dir, pg_cmd)?;

    // panes[3] => Redis logs (best-effort)
    let redis_cmd = r#"docker ps --format '{{.Names}}' | grep -Ei '^redis' | head -n1 | xargs -r docker logs -f || echo 'No redis'"#;
    send_to_target_sh(tmux, &panes[2], proj_dir, redis_cmd)?;

    // panes[4] => spare/interactive
    send_to_target_sh(tmux, &panes[3], proj_dir, "exec $SHELL")?;
    Ok(())
}
