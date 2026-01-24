use std::env;
use std::process::{exit, Command};

use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct DnsResult {
    timestamp: String,
    server: String,
    query: String,
    query_type: String,
    status: String,
    time_ms: Option<u32>,
    answers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct Args {
    server: Option<String>,
    query: String,
    query_type: String,
    timeout: u32,
}

fn print_usage() {
    eprintln!(
        "dig-json {}

DNS lookup with JSON output.

USAGE:
    dig-json [OPTIONS] <DOMAIN>

ARGUMENTS:
    <DOMAIN>       Domain name to query

OPTIONS:
    -s <SERVER>    DNS server to query (default: system resolver)
    -t <TYPE>      Query type: A, AAAA, CNAME, MX, TXT (default: A)
    -W <TIMEOUT>   Timeout in seconds (default: 5)
    -h, --help     Print help
    -V, --version  Print version",
        env!("CARGO_PKG_VERSION")
    );
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return Err("missing domain".to_string());
    }

    let mut server: Option<String> = None;
    let mut query: Option<String> = None;
    let mut query_type = "A".to_string();
    let mut timeout: u32 = 5;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                exit(0);
            }
            "-V" | "--version" => {
                println!("dig-json {}", env!("CARGO_PKG_VERSION"));
                exit(0);
            }
            "-s" => {
                i += 1;
                server = Some(
                    args.get(i)
                        .ok_or("-s requires a value")?
                        .clone(),
                );
            }
            "-t" => {
                i += 1;
                query_type = args
                    .get(i)
                    .ok_or("-t requires a value")?
                    .to_uppercase();
            }
            "-W" => {
                i += 1;
                timeout = args
                    .get(i)
                    .ok_or("-W requires a value")?
                    .parse()
                    .map_err(|_| "invalid timeout")?;
            }
            arg if arg.starts_with('-') => {
                return Err(format!("unknown option: {}", arg));
            }
            arg => {
                if query.is_some() {
                    return Err("unexpected argument".to_string());
                }
                query = Some(arg.to_string());
            }
        }
        i += 1;
    }

    let query = query.ok_or("missing domain")?;

    Ok(Args {
        server,
        query,
        query_type,
        timeout,
    })
}

fn parse_dig_output(output: &str) -> (String, Option<u32>, Vec<String>) {
    let mut status = "unknown".to_string();
    let mut time_ms = None;
    let mut answers = Vec::new();

    for line in output.lines() {
        if line.contains("status:") {
            if let Some(s) = line.split("status:").nth(1) {
                if let Some(s) = s.split(',').next() {
                    status = s.trim().to_lowercase();
                }
            }
        }

        if line.contains("Query time:") {
            if let Some(s) = line.split("Query time:").nth(1) {
                if let Some(s) = s.trim().split_whitespace().next() {
                    time_ms = s.parse().ok();
                }
            }
        }

        // Parse answer section lines (not comments, not empty)
        if !line.starts_with(';') && !line.is_empty() && line.contains('\t') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                // Format: name ttl class type value
                let record_type = parts[3];
                let value = parts[4..].join(" ");
                if record_type == "A" || record_type == "AAAA" || record_type == "CNAME"
                    || record_type == "MX" || record_type == "TXT" {
                    answers.push(value);
                }
            }
        }
    }

    (status, time_ms, answers)
}

fn run_dig(args: &Args) -> DnsResult {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let server_display = args.server.clone().unwrap_or_else(|| "system".to_string());

    let mut cmd = Command::new("dig");

    if let Some(ref server) = args.server {
        cmd.arg(format!("@{}", server));
    }

    cmd.arg(&args.query);
    cmd.arg(&args.query_type);
    cmd.arg(format!("+time={}", args.timeout));
    cmd.arg("+tries=1");

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return DnsResult {
                timestamp,
                server: server_display,
                query: args.query.clone(),
                query_type: args.query_type.clone(),
                status: "error".to_string(),
                time_ms: None,
                answers: vec![],
                error: Some(format!("failed to run dig: {}", e)),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        return DnsResult {
            timestamp,
            server: server_display,
            query: args.query.clone(),
            query_type: args.query_type.clone(),
            status: "error".to_string(),
            time_ms: None,
            answers: vec![],
            error: Some("dig command failed".to_string()),
        };
    }

    let (status, time_ms, answers) = parse_dig_output(&stdout);

    DnsResult {
        timestamp,
        server: server_display,
        query: args.query.clone(),
        query_type: args.query_type.clone(),
        status,
        time_ms,
        answers,
        error: None,
    }
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("Run 'dig-json --help' for usage.");
            exit(2);
        }
    };

    let result = run_dig(&args);
    let success = result.status == "noerror" && result.error.is_none();

    if let Ok(json) = serde_json::to_string(&result) {
        println!("{}", json);
    }

    exit(if success { 0 } else { 1 });
}
