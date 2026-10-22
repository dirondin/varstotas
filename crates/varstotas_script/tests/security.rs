//! Regressions for capability boundaries and bounded execution.
use std::time::{Duration, Instant};
use varstotas_kernel::{GenValues, Model, glam::DMat4};
use varstotas_script::{ScriptCtx, generator, run};

#[test]
fn default_context_cannot_use_host_io() {
    for code in [
        r#"read_text("missing")"#,
        r#"write_text("missing", "x")"#,
        r#"fetch_text("http://127.0.0.1/")"#,
        r#"fetch_file("file:///x", "missing")"#,
        r#"file_exists(".")"#,
        r#"list_files(".", "")"#,
        r#"list_dirs(".")"#,
    ] {
        let r = run(&mut Model::new(), &ScriptCtx::default(), code);
        assert!(r.error.is_some(), "IO unexpectedly available: {code}");
    }
}

#[test]
fn document_generator_cannot_inherit_trusted_io() {
    let path =
        std::env::temp_dir().join(format!("varstotas-denied-generator-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let src = format!(
        "// @generator Denied\nfn build(p) {{ write_text({}, \"bad\"); }}",
        serde_json::to_string(&path.to_string_lossy()).unwrap()
    );
    let mut m = Model::new();
    generator::add(&mut m, &src).unwrap();
    assert!(generator::place(&mut m, &[], "Denied", &GenValues::new(), DMat4::IDENTITY).is_err());
    let r = run(
        &mut m,
        &ScriptCtx {
            trusted_io: true,
            ..Default::default()
        },
        r#"generate("Denied", ORIGIN, #{})"#,
    );
    assert!(r.error.as_deref().is_some_and(|e| e.contains("write_text")), "{:?}", r.error);
    assert!(!path.exists());
}

#[test]
fn explicitly_trusted_local_script_can_use_files() {
    let p = std::env::temp_dir().join(format!("varstotas-trusted-{}", std::process::id()));
    let code = format!(
        "write_text({0}, \"ok\"); read_text({0})",
        serde_json::to_string(&p.to_string_lossy()).unwrap()
    );
    let r = run(
        &mut Model::new(),
        &ScriptCtx {
            trusted_io: true,
            ..Default::default()
        },
        &code,
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "ok");
    std::fs::remove_file(p).unwrap();
}

#[test]
fn curl_options_and_local_schemes_are_rejected() {
    for url in [
        "-K/tmp/config",
        "file:///etc/hosts",
        "ftp://example.org/x",
        "http://x/\n",
    ] {
        assert!(
            varstotas_script::network::fetch(url, &[], Duration::from_secs(1), || false).is_err()
        );
    }
}

#[test]
fn native_download_respects_script_deadline() {
    use std::io::Read;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
                _ => return,
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let _ = stream.read(&mut [0; 4096]);
        std::thread::sleep(Duration::from_millis(300));
    });
    let started = Instant::now();
    let r = run(
        &mut Model::new(),
        &ScriptCtx {
            trusted_io: true,
            time_limit: Some(Duration::from_millis(80)),
            ..Default::default()
        },
        &format!("fetch_text(\"http://127.0.0.1:{port}/\")"),
    );
    assert!(r.error.is_some());
    assert!(started.elapsed() < Duration::from_secs(2));
    server.join().unwrap();
}

#[test]
fn source_and_expression_allocation_are_bounded() {
    assert!(
        run(
            &mut Model::new(),
            &ScriptCtx::default(),
            &" ".repeat(1024 * 1024 + 1)
        )
        .error
        .is_some()
    );
    assert!(
        run(
            &mut Model::new(),
            &ScriptCtx::default(),
            "let s = \"0123456789\"; loop { s += s; }"
        )
        .error
        .is_some()
    );
}

#[cfg(unix)]
#[test]
fn child_process_output_and_time_are_bounded() {
    let mut cmd = std::process::Command::new("sh");
    cmd.args(["-c", "while :; do printf '1234567890'; done"]);
    assert!(
        varstotas_script::process::output(&mut cmd, Duration::from_secs(2), 4096, || false)
            .is_err()
    );
    let started = Instant::now();
    let mut cmd = std::process::Command::new("sh");
    cmd.args(["-c", "sleep 30 & wait"]);
    assert!(
        varstotas_script::process::output(&mut cmd, Duration::from_millis(50), 4096, || false)
            .is_err()
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}
