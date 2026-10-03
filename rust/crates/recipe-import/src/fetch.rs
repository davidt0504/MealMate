//! The network boundary (OPT-001 design §3): which URLs may be fetched, which addresses may
//! be connected to, and the limits on one page. Everything here treats the network as
//! hostile; nothing is retried and nothing about a page is logged.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ureq::config::Config;
use ureq::http::Uri;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use ureq::Agent;
use url::{Host, Url};

use crate::ImportError;

/// Per-page limits. `Limits::default()` is the only production value; the relaxed knobs
/// exist only for tests that talk to a local server.
#[derive(Debug, Clone)]
pub struct Limits {
    pub(crate) total: Duration,
    pub(crate) connect: Duration,
    pub(crate) max_bytes: u64,
    pub(crate) max_redirects: u8,
    pub(crate) allow_loopback: bool,
    pub(crate) allow_any_port: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            total: Duration::from_secs(30),
            connect: Duration::from_secs(10),
            max_bytes: 3 * 1024 * 1024,
            max_redirects: 5,
            allow_loopback: false,
            allow_any_port: false,
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Limits {
    /// Loopback and any port allowed, so a test can serve pages from `127.0.0.1:<random>`
    /// through an injected resolver. Never reachable from a shipped build.
    pub fn for_local_tests(total: Duration) -> Self {
        Self {
            total,
            connect: total,
            allow_loopback: true,
            allow_any_port: true,
            ..Self::default()
        }
    }
}

/// A fetched HTML page, after redirects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedPage {
    pub final_url: Url,
    pub html: String,
}

/// Parses a user- or page-supplied URL and refuses anything that is not a plain public web
/// address. The path and query are left exactly as given — this is the URL that is fetched,
/// and rewriting it (say, dropping a trailing slash a site redirects back to) can loop.
pub fn guard_url(raw: &str, limits: &Limits) -> Result<Url, ImportError> {
    let mut url = Url::parse(raw.trim()).map_err(|_| ImportError::Blocked)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ImportError::Blocked);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ImportError::Blocked);
    }
    if !limits.allow_any_port && url.port().is_some_and(|p| p != 80 && p != 443) {
        return Err(ImportError::Blocked);
    }
    let host = match url.host() {
        Some(Host::Domain(d)) => d.trim_end_matches('.').to_owned(),
        _ => return Err(ImportError::Blocked),
    };
    if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        return Err(ImportError::Blocked);
    }
    url.set_host(Some(&host))
        .map_err(|_| ImportError::Blocked)?;
    // Checked after the trailing dots are gone: the url crate parses `2130706433`, `0x7f.1`
    // and `[::1]` as IP hosts, and `127.0.0.1..` only becomes one once trimmed, so requiring
    // `Host::Domain` here refuses every IP-literal spelling.
    if !matches!(url.host(), Some(Host::Domain(_))) {
        return Err(ImportError::Blocked);
    }
    url.set_fragment(None);
    Ok(url)
}

/// [`guard_url`] plus the duplicate-check normalization (design §8): `utm_*` parameters and
/// a trailing slash on a non-root path dropped (scheme and host are already lowercased by
/// the url crate). This is what is stored as a recipe's `source_url`, never what is fetched.
pub fn normalize_url(raw: &str, limits: &Limits) -> Result<Url, ImportError> {
    let mut url = guard_url(raw, limits)?;
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| !k.starts_with("utm_"))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(kept);
    }
    let path = url.path().to_owned();
    if path.len() > 1 && path.ends_with('/') {
        url.set_path(path.trim_end_matches('/'));
    }
    Ok(url)
}

/// Whether an address is on the public internet. An allowlist for IPv6 (`2000::/3` only)
/// and a denylist for IPv4; every IPv6 form that embeds an IPv4 address is judged by that
/// address, so a private target can't hide inside a mapped, compatible, NAT64 or 6to4 form.
pub fn is_global(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_global_v4(v4),
        IpAddr::V6(v6) => is_global_v6(v6),
    }
}

fn is_global_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..128).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..32).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224)
}

fn is_global_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    let embedded =
        |hi: u16, lo: u16| Ipv4Addr::new((hi >> 8) as u8, hi as u8, (lo >> 8) as u8, lo as u8);
    // IPv4-mapped ::ffff:a.b.c.d and IPv4-compatible ::a.b.c.d.
    if s[..5] == [0; 5] && (s[5] == 0xffff || s[5] == 0) {
        return is_global_v4(embedded(s[6], s[7]));
    }
    // NAT64 64:ff9b::/96.
    if s[0] == 0x64 && s[1] == 0xff9b && s[2..6] == [0; 4] {
        return is_global_v4(embedded(s[6], s[7]));
    }
    // 6to4 2002::/16 carries its IPv4 in bits 16..48.
    if s[0] == 0x2002 {
        return is_global_v4(embedded(s[1], s[2]));
    }
    let global_unicast = (s[0] & 0xe000) == 0x2000;
    let documentation = s[0] == 0x2001 && s[1] == 0x0db8;
    let teredo = s[0] == 0x2001 && s[1] == 0;
    global_unicast && !documentation && !teredo
}

/// Resolves through `inner`, then refuses the hop unless **every** answer is public, and
/// returns exactly the answers it checked — so the address that is connected to is always
/// one that was validated (no DNS-rebinding window). TLS still verifies the URI's host.
#[derive(Debug)]
struct GuardResolver {
    inner: Arc<dyn Resolver>,
    allow_loopback: bool,
    /// Set when a hop was refused for its address, so the caller can tell `Blocked` from
    /// an ordinary lookup failure (ureq only lets a resolver return `HostNotFound`).
    blocked: Arc<Mutex<bool>>,
}

impl Resolver for GuardResolver {
    fn resolve(
        &self,
        uri: &Uri,
        config: &Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let addrs = self.inner.resolve(uri, config, timeout)?;
        let allowed = |ip: IpAddr| is_global(ip) || (self.allow_loopback && ip.is_loopback());
        if addrs.is_empty() || !addrs.iter().all(|a| allowed(a.ip())) {
            *self.blocked.lock().unwrap_or_else(|e| e.into_inner()) = true;
            return Err(ureq::Error::HostNotFound);
        }
        Ok(addrs)
    }
}

/// The production lookup: ureq's own resolver, which bounds `getaddrinfo` by the request's
/// timeout on a helper thread.
pub fn system_dns() -> Arc<dyn Resolver> {
    Arc::new(DefaultResolver::default())
}

fn agent(limits: &Limits, remaining: Duration, resolver: GuardResolver) -> Agent {
    let config = Agent::config_builder()
        // Never a proxy from the environment: the guard would then only ever see the
        // proxy's host, never the page's.
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_connect(Some(limits.connect.min(remaining)))
        .timeout_global(Some(remaining))
        .user_agent(concat!(
            "Kimatta/",
            env!("CARGO_PKG_VERSION"),
            " (recipe import)"
        ))
        .build();
    Agent::with_parts(config, DefaultConnector::default(), resolver)
}

/// Fetches one page under `limits`, following at most `max_redirects` redirects by hand so
/// every hop is normalized and address-checked again. One deadline covers every hop.
///
/// ponytail: no cancellation token — a superseded import is dropped by the Dart side and this
/// call ends by its own deadline (30 s; a timed-out `getaddrinfo` helper thread lingers until
/// the OS returns, at most one per hop). Add a token if orphaned fetches ever starve the
/// bridge's worker pool.
pub fn fetch_page(
    raw_url: &str,
    limits: &Limits,
    dns: Arc<dyn Resolver>,
) -> Result<FetchedPage, ImportError> {
    let deadline = Instant::now() + limits.total;
    let mut url = guard_url(raw_url, limits)?;
    for hop in 0..=limits.max_redirects {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ImportError::Timeout);
        }
        let blocked = Arc::new(Mutex::new(false));
        let guard = GuardResolver {
            inner: dns.clone(),
            allow_loopback: limits.allow_loopback,
            blocked: blocked.clone(),
        };
        let response = agent(limits, remaining, guard)
            .get(url.as_str())
            .header("Accept", "text/html")
            .call();
        let mut response = match response {
            Ok(r) => r,
            Err(e) if *blocked.lock().unwrap_or_else(|p| p.into_inner()) => {
                drop(e);
                return Err(ImportError::Blocked);
            }
            Err(e) => return Err(transport_error(e)),
        };
        let status = response.status().as_u16();
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            if hop == limits.max_redirects {
                return Err(ImportError::Unreadable);
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or(ImportError::Unreadable)?;
            let next = url.join(location).map_err(|_| ImportError::Blocked)?;
            url = guard_url(next.as_str(), limits)?;
            continue;
        }
        match status {
            401 | 403 | 429 | 451 => return Err(ImportError::Refused),
            500..=599 => return Err(ImportError::Offline),
            200..=299 => {}
            _ => return Err(ImportError::Unreadable),
        }
        let html_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|v| {
                v.split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase()
            })
            .is_some_and(|t| t == "text/html" || t == "application/xhtml+xml");
        if !html_type {
            return Err(ImportError::Unreadable);
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit(limits.max_bytes)
            .read_to_vec()
            .map_err(transport_error)?;
        // Pages that lie about their charset still import; bad bytes become U+FFFD.
        let html = String::from_utf8_lossy(&bytes).into_owned();
        return Ok(FetchedPage {
            final_url: url,
            html,
        });
    }
    Err(ImportError::Unreadable)
}

fn transport_error(e: ureq::Error) -> ImportError {
    match e {
        ureq::Error::Timeout(_) => ImportError::Timeout,
        ureq::Error::BodyExceedsLimit(_)
        | ureq::Error::LargeResponseHeader(..)
        | ureq::Error::Protocol(_)
        | ureq::Error::Decompress(..) => ImportError::Unreadable,
        ureq::Error::Io(io) if io.kind() == std::io::ErrorKind::TimedOut => ImportError::Timeout,
        _ => ImportError::Offline,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::thread;

    /// Answers every lookup with fixed addresses, so no test ever asks the system DNS.
    #[derive(Debug)]
    pub(crate) struct FixedDns(pub Vec<IpAddr>);

    impl Resolver for FixedDns {
        fn resolve(
            &self,
            uri: &Uri,
            _config: &Config,
            _timeout: NextTimeout,
        ) -> Result<ResolvedSocketAddrs, ureq::Error> {
            let port = uri.port_u16().unwrap_or(80);
            let mut out = self.empty();
            for ip in &self.0 {
                out.push(SocketAddr::new(*ip, port));
            }
            Ok(out)
        }
    }

    /// Answers by host name: `public.test` → loopback (the local server), anything else →
    /// the private address, so a redirect to another host meets a private answer.
    #[derive(Debug)]
    struct SplitDns;

    impl Resolver for SplitDns {
        fn resolve(
            &self,
            uri: &Uri,
            _config: &Config,
            _timeout: NextTimeout,
        ) -> Result<ResolvedSocketAddrs, ureq::Error> {
            let port = uri.port_u16().unwrap_or(80);
            let ip: IpAddr = if uri.host() == Some("public.test") {
                Ipv4Addr::LOCALHOST.into()
            } else {
                Ipv4Addr::new(10, 0, 0, 7).into()
            };
            let mut out = self.empty();
            out.push(SocketAddr::new(ip, port));
            Ok(out)
        }
    }

    pub(crate) fn loopback() -> Arc<dyn Resolver> {
        Arc::new(FixedDns(vec![Ipv4Addr::LOCALHOST.into()]))
    }

    /// A one-thread HTTP server: `respond(path)` gives the raw response for each request.
    pub(crate) fn serve(
        requests: usize,
        respond: impl Fn(&str) -> Vec<u8> + Send + 'static,
    ) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            for stream in listener.incoming().take(requests) {
                let Ok(mut stream) = stream else { return };
                let path = read_path(&mut stream);
                let _ = stream.write_all(&respond(&path));
            }
        });
        port
    }

    fn read_path(stream: &mut TcpStream) -> String {
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let head = String::from_utf8_lossy(&buf[..n]);
        head.split_whitespace().nth(1).unwrap_or("/").to_owned()
    }

    pub(crate) fn html(body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    fn status(code: u16, extra: &str) -> Vec<u8> {
        format!("HTTP/1.1 {code} X\r\n{extra}Content-Length: 0\r\nConnection: close\r\n\r\n")
            .into_bytes()
    }

    fn local(total_ms: u64) -> Limits {
        Limits::for_local_tests(Duration::from_millis(total_ms))
    }

    // --- URL guard: standard, edge, adversarial ----------------------------------------------

    #[test]
    fn a_public_https_url_is_accepted_and_normalized() {
        let url = normalize_url(
            "https://Example.COM/Recipes/Shawarma/?utm_source=x&id=3#comments",
            &Limits::default(),
        )
        .unwrap();
        assert_eq!(url.as_str(), "https://example.com/Recipes/Shawarma?id=3");
    }

    #[test]
    fn http_default_ports_idn_and_trailing_dot_are_accepted() {
        let d = Limits::default();
        assert_eq!(
            normalize_url("http://example.com/", &d).unwrap().as_str(),
            "http://example.com/"
        );
        assert_eq!(
            normalize_url("https://example.com:443/a", &d)
                .unwrap()
                .as_str(),
            "https://example.com/a"
        );
        assert_eq!(
            normalize_url("https://bücher.example/a", &d)
                .unwrap()
                .host_str(),
            Some("xn--bcher-kva.example")
        );
        assert_eq!(
            normalize_url("https://example.com./a", &d)
                .unwrap()
                .host_str(),
            Some("example.com")
        );
        assert_eq!(
            normalize_url("https://example.com/a?utm_medium=x", &d)
                .unwrap()
                .query(),
            None
        );
    }

    #[test]
    fn unsafe_urls_are_blocked() {
        let d = Limits::default();
        let bad = [
            "https://user:pass@example.com/",
            "https://user@example.com/",
            "file:///etc/passwd",
            "ftp://example.com/",
            "javascript:alert(1)",
            "https://2130706433/",
            "https://0x7f.1/",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://localhost/",
            "https://api.localhost/",
            "https://example.com:8080/",
            "not a url",
        ];
        assert!(!bad.is_empty());
        for raw in bad {
            assert_eq!(normalize_url(raw, &d), Err(ImportError::Blocked), "{raw}");
        }
    }

    #[test]
    fn only_public_addresses_are_global() {
        let private = [
            "0.0.0.0",
            "0.1.2.3",
            "10.0.0.1",
            "100.64.0.1",
            "100.127.255.255",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.19.255.255",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "fe80::1",
            "fc00::1",
            "fd12::1",
            "fec0::1",
            "ff02::1",
            "::ffff:10.0.0.1",
            "::ffff:127.0.0.1",
            "::10.0.0.1",
            "64:ff9b::a00:1",
            "2002:a00:1::",
            "2001:db8::1",
            "2001::1",
        ];
        let public = [
            "93.184.216.34",
            "1.1.1.1",
            "100.128.0.1",
            "172.32.0.1",
            "2606:4700::1111",
            "::ffff:93.184.216.34",
            "64:ff9b::5db8:d822",
            "2002:5db8:d822::",
        ];
        assert!(!private.is_empty() && !public.is_empty());
        for ip in private {
            assert!(!is_global(ip.parse().unwrap()), "{ip} must not be global");
        }
        for ip in public {
            assert!(is_global(ip.parse().unwrap()), "{ip} must be global");
        }
    }

    #[test]
    fn production_limits_never_allow_loopback_or_odd_ports() {
        let d = Limits::default();
        assert!(!d.allow_loopback && !d.allow_any_port);
        assert_eq!(d.max_redirects, 5);
        assert_eq!(d.max_bytes, 3 * 1024 * 1024);
        assert_eq!(d.total, Duration::from_secs(30));
    }

    #[test]
    fn the_agent_never_uses_an_environment_proxy() {
        let guard = GuardResolver {
            inner: loopback(),
            allow_loopback: false,
            blocked: Arc::default(),
        };
        let agent = agent(&Limits::default(), Duration::from_secs(1), guard);
        assert!(agent.config().proxy().is_none());
        assert_eq!(agent.config().max_redirects(), 0);
        assert!(!agent.config().http_status_as_error());
    }

    // --- Fetch against a local server ---------------------------------------------------------

    #[test]
    fn a_page_is_fetched_through_a_redirect() {
        let port = serve(2, |path| {
            if path == "/old" {
                status(301, "Location: /new\r\n")
            } else {
                html("<p>recipe</p>")
            }
        });
        let page = fetch_page(
            &format!("http://public.test:{port}/old"),
            &local(5000),
            loopback(),
        )
        .unwrap();
        assert_eq!(page.html, "<p>recipe</p>");
        assert_eq!(page.final_url.path(), "/new");
    }

    #[test]
    fn a_redirect_that_adds_a_trailing_slash_is_followed_not_looped() {
        // A real site 301s `/recipe` to `/recipe/`; stripping the slash again before the
        // next hop looped until the redirect cap (seen on-device 2026-10-02).
        let port = serve(2, |path| {
            if path == "/recipe" {
                status(301, "Location: /recipe/\r\n")
            } else {
                html("<p>ok</p>")
            }
        });
        let page = fetch_page(
            &format!("http://public.test:{port}/recipe/?utm_source=x"),
            &local(5000),
            loopback(),
        )
        .unwrap();
        assert_eq!(page.final_url.path(), "/recipe/");
        assert_eq!(page.html, "<p>ok</p>");
    }

    #[test]
    fn an_ip_literal_hidden_behind_trailing_dots_is_blocked() {
        let d = Limits::default();
        for raw in [
            "https://127.0.0.1../",
            "https://0x7f.0.0.1../",
            "https://10.0.0.1.%2e/",
            "https://8.8.8.8../",
        ] {
            assert_eq!(normalize_url(raw, &d), Err(ImportError::Blocked), "{raw}");
        }
    }

    #[test]
    fn production_limits_refuse_a_loopback_answer() {
        let err = fetch_page("https://public.test/", &Limits::default(), loopback()).unwrap_err();
        assert_eq!(err, ImportError::Blocked);
    }

    #[test]
    fn a_redirect_whose_dns_answer_turns_private_is_blocked() {
        let port = serve(1, move |_| {
            status(302, "Location: http://intranet.test/admin\r\n")
        });
        let mut limits = local(5000);
        limits.allow_loopback = true;
        let err = fetch_page(
            &format!("http://public.test:{port}/"),
            &limits,
            Arc::new(SplitDns),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Blocked);
    }

    #[test]
    fn a_sixth_redirect_is_refused() {
        let port = serve(6, |path| {
            let n: u32 = path.trim_start_matches("/r").parse().unwrap_or(0);
            status(302, &format!("Location: /r{}\r\n", n + 1))
        });
        let err = fetch_page(
            &format!("http://public.test:{port}/r0"),
            &local(5000),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Unreadable);
    }

    #[test]
    fn a_body_over_the_cap_is_unreadable() {
        let body = "a".repeat(3 * 1024 * 1024 + 1);
        let port = serve(1, move |_| html(&body));
        let err = fetch_page(
            &format!("http://public.test:{port}/"),
            &local(10000),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Unreadable);
    }

    #[test]
    fn a_slow_drip_hits_the_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            if let Some(Ok(mut s)) = listener.incoming().next() {
                read_path(&mut s);
                let _ = s.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 100\r\n\r\n",
                );
                for _ in 0..100 {
                    if s.write_all(b"a").is_err() {
                        return;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        });
        let started = Instant::now();
        let err = fetch_page(
            &format!("http://public.test:{port}/"),
            &local(400),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Timeout);
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn statuses_map_to_their_failure_kinds() {
        let cases = [
            (403, ImportError::Refused),
            (401, ImportError::Refused),
            (429, ImportError::Refused),
            (451, ImportError::Refused),
            (404, ImportError::Unreadable),
            (410, ImportError::Unreadable),
            (503, ImportError::Offline),
        ];
        for (code, expected) in cases {
            let port = serve(1, move |_| status(code, ""));
            let err = fetch_page(
                &format!("http://public.test:{port}/"),
                &local(5000),
                loopback(),
            )
            .unwrap_err();
            assert_eq!(err, expected, "status {code}");
        }
    }

    #[test]
    fn a_non_html_page_is_unreadable() {
        let port = serve(1, |_| {
            b"HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: 3\r\nConnection: close\r\n\r\n%PD".to_vec()
        });
        let err = fetch_page(
            &format!("http://public.test:{port}/"),
            &local(5000),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Unreadable);
    }

    #[test]
    fn a_non_utf8_body_is_decoded_lossily() {
        let port = serve(1, |_| {
            let mut r = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 4\r\nConnection: close\r\n\r\n".to_vec();
            r.extend_from_slice(&[b'c', 0xff, b'a', b'f']);
            r
        });
        let page = fetch_page(
            &format!("http://public.test:{port}/"),
            &local(5000),
            loopback(),
        )
        .unwrap();
        assert_eq!(page.html, "c\u{fffd}af");
    }

    #[test]
    fn an_unreachable_host_is_offline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let err = fetch_page(
            &format!("http://public.test:{port}/"),
            &local(2000),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Offline);
    }
}
