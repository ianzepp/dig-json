# dig-json

DNS lookup with JSON output.

## Usage

```bash
# Query using system resolver
dig-json apple.com
{"timestamp":"2026-01-24T12:34:56.789Z","server":"system","query":"apple.com","query_type":"A","status":"noerror","time_ms":42,"answers":["17.253.144.10"]}

# Query specific DNS server
dig-json -s 8.8.8.8 google.com

# Query different record type
dig-json -t AAAA google.com
dig-json -t MX gmail.com
```

## Output Format

```typescript
interface DnsResult {
  timestamp: string;      // ISO 8601
  server: string;         // DNS server used, or "system"
  query: string;          // Domain queried
  query_type: string;     // A, AAAA, CNAME, MX, TXT
  status: string;         // noerror, nxdomain, servfail, timeout, etc.
  time_ms: number | null; // Query time in milliseconds
  answers: string[];      // Resolved values
  error?: string;         // Error message if failed
}
```

## CLI Reference

```
dig-json [OPTIONS] <DOMAIN>

Arguments:
  <DOMAIN>       Domain name to query

Options:
  -s <SERVER>    DNS server to query (default: system resolver)
  -t <TYPE>      Query type: A, AAAA, CNAME, MX, TXT (default: A)
  -W <TIMEOUT>   Timeout in seconds (default: 5)
  -h, --help     Print help
  -V, --version  Print version
```

## Exit Codes

- `0` — Query successful (status: noerror)
- `1` — Query failed (nxdomain, timeout, error)
- `2` — Invalid arguments

## Building

```bash
cargo build --release
```

## Installing

```bash
./install.sh
```

## License

MIT
