use std::{str::FromStr, time::Duration};

use mail_parser::HeaderName;
use nlp::language::Language;
use store::rand::{distributions::Alphanumeric, thread_rng, Rng};
use utils::config::{cron::SimpleCron, utils::ParseValue, Config, Rate};

use super::capabilities::BaseCapabilities;

pub struct JmapConfig {
    pub default_language: Language,
    pub query_max_results: usize,
    pub changes_max_results: usize,
    pub snippet_max_results: usize,

    pub request_max_size: usize,
    pub request_max_calls: usize,
    pub request_max_concurrent: u64,

    pub get_max_objects: usize,
    pub set_max_objects: usize,

    pub upload_max_size: usize,
    pub upload_max_concurrent: u64,

    pub upload_tmp_quota_size: usize,
    pub upload_tmp_quota_amount: usize,
    pub upload_tmp_ttl: u64,

    pub mailbox_max_depth: usize,
    pub mailbox_name_max_len: usize,
    pub mail_attachments_max_size: usize,
    pub mail_parse_max_items: usize,
    pub mail_max_size: usize,

    pub sieve_max_script_name: usize,
    pub sieve_max_scripts: usize,

    pub session_cache_ttl: Duration,
    pub rate_authenticated: Option<Rate>,
    pub rate_authenticate_req: Option<Rate>,
    pub rate_anonymous: Option<Rate>,
    pub rate_use_forwarded: bool,

    pub event_source_throttle: Duration,
    pub push_max_total: usize,
    pub push_attempt_interval: Duration,
    pub push_attempts_max: u32,
    pub push_retry_interval: Duration,
    pub push_timeout: Duration,
    pub push_verify_timeout: Duration,
    pub push_throttle: Duration,

    pub web_socket_throttle: Duration,
    pub web_socket_timeout: Duration,
    pub web_socket_heartbeat: Duration,

    pub oauth_key: String,
    pub oauth_expiry_user_code: u64,
    pub oauth_expiry_auth_code: u64,
    pub oauth_expiry_token: u64,
    pub oauth_expiry_refresh_token: u64,
    pub oauth_expiry_refresh_token_renew: u64,
    pub oauth_max_auth_attempts: u32,

    pub spam_header: Option<(HeaderName<'static>, String)>,

    pub http_headers: Vec<(hyper::header::HeaderName, hyper::header::HeaderValue)>,

    pub encrypt: bool,
    pub encrypt_append: bool,

    pub principal_allow_lookups: bool,

    pub capabilities: BaseCapabilities,
    pub session_purge_frequency: SimpleCron,
}

impl JmapConfig {
    pub fn parse(config: &mut Config) -> Self {
        let mut jmap = JmapConfig {
            default_language: Language::from_iso_639(
                config
                    .value("storage.full-text.default-language")
                    .unwrap_or("en"),
            )
            .unwrap_or(Language::English),
            query_max_results: config
                .property_("jmap.protocol.query.max-results")
                .unwrap_or(5000),
            changes_max_results: config
                .property_("jmap.protocol.changes.max-results")
                .unwrap_or(5000),
            snippet_max_results: config
                .property_("jmap.protocol.search-snippet.max-results")
                .unwrap_or(100),
            request_max_size: config
                .property_("jmap.protocol.request.max-size")
                .unwrap_or(10000000),
            request_max_calls: config
                .property_("jmap.protocol.request.max-calls")
                .unwrap_or(16),
            request_max_concurrent: config
                .property_("jmap.protocol.request.max-concurrent")
                .unwrap_or(4),
            get_max_objects: config
                .property_("jmap.protocol.get.max-objects")
                .unwrap_or(500),
            set_max_objects: config
                .property_("jmap.protocol.set.max-objects")
                .unwrap_or(500),
            upload_max_size: config
                .property_("jmap.protocol.upload.max-size")
                .unwrap_or(50000000),
            upload_max_concurrent: config
                .property_("jmap.protocol.upload.max-concurrent")
                .unwrap_or(4),
            upload_tmp_quota_size: config
                .property_("jmap.protocol.upload.quota.size")
                .unwrap_or(50000000),
            upload_tmp_quota_amount: config
                .property_("jmap.protocol.upload.quota.files")
                .unwrap_or(1000),
            upload_tmp_ttl: config
                .property_or_default_::<Duration>("jmap.protocol.upload.ttl", "1h")
                .unwrap_or_else(|| Duration::from_secs(3600))
                .as_secs(),
            mailbox_max_depth: config.property_("jmap.mailbox.max-depth").unwrap_or(10),
            mailbox_name_max_len: config
                .property_("jmap.mailbox.max-name-length")
                .unwrap_or(255),
            mail_attachments_max_size: config
                .property_("jmap.email.max-attachment-size")
                .unwrap_or(50000000),
            mail_max_size: config.property_("jmap.email.max-size").unwrap_or(75000000),
            mail_parse_max_items: config.property_("jmap.email.parse.max-items").unwrap_or(10),
            sieve_max_script_name: config
                .property_("sieve.untrusted.limits.name-length")
                .unwrap_or(512),
            sieve_max_scripts: config
                .property_("sieve.untrusted.limits.max-scripts")
                .unwrap_or(256),
            capabilities: BaseCapabilities::default(),
            session_cache_ttl: config
                .property_("cache.session.ttl")
                .unwrap_or(Duration::from_secs(3600)),
            rate_authenticated: config.property_or_default_("jmap.rate-limit.account", "1000/1m"),
            rate_authenticate_req: config
                .property_or_default_("authentication.rate-limit", "10/1m"),
            rate_anonymous: config.property_or_default_("jmap.rate-limit.anonymous", "100/1m"),
            rate_use_forwarded: config
                .property_("jmap.rate-limit.use-forwarded")
                .unwrap_or(false),
            oauth_key: config
                .value("oauth.key")
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    thread_rng()
                        .sample_iter(Alphanumeric)
                        .take(64)
                        .map(char::from)
                        .collect::<String>()
                }),
            oauth_expiry_user_code: config
                .property_or_default_::<Duration>("oauth.expiry.user-code", "30m")
                .unwrap_or_else(|| Duration::from_secs(30 * 60))
                .as_secs(),
            oauth_expiry_auth_code: config
                .property_or_default_::<Duration>("oauth.expiry.auth-code", "10m")
                .unwrap_or_else(|| Duration::from_secs(10 * 60))
                .as_secs(),
            oauth_expiry_token: config
                .property_or_default_::<Duration>("oauth.expiry.token", "1h")
                .unwrap_or_else(|| Duration::from_secs(60 * 60))
                .as_secs(),
            oauth_expiry_refresh_token: config
                .property_or_default_::<Duration>("oauth.expiry.refresh-token", "30d")
                .unwrap_or_else(|| Duration::from_secs(30 * 24 * 60 * 60))
                .as_secs(),
            oauth_expiry_refresh_token_renew: config
                .property_or_default_::<Duration>("oauth.expiry.refresh-token-renew", "4d")
                .unwrap_or_else(|| Duration::from_secs(4 * 24 * 60 * 60))
                .as_secs(),
            oauth_max_auth_attempts: config
                .property_or_default_("oauth.auth.max-attempts", "3")
                .unwrap_or(10),
            event_source_throttle: config
                .property_or_default_("jmap.event-source.throttle", "1s")
                .unwrap_or_else(|| Duration::from_secs(1)),
            web_socket_throttle: config
                .property_or_default_("jmap.web-socket.throttle", "1s")
                .unwrap_or_else(|| Duration::from_secs(1)),
            web_socket_timeout: config
                .property_or_default_("jmap.web-socket.timeout", "10m")
                .unwrap_or_else(|| Duration::from_secs(10 * 60)),
            web_socket_heartbeat: config
                .property_or_default_("jmap.web-socket.heartbeat", "1m")
                .unwrap_or_else(|| Duration::from_secs(60)),
            push_max_total: config
                .property_or_default_("jmap.push.max-total", "100")
                .unwrap_or(100),
            principal_allow_lookups: config
                .property_("jmap.principal.allow-lookups")
                .unwrap_or(true),
            encrypt: config
                .property_or_default_("storage.encryption.enable", "true")
                .unwrap_or(true),
            encrypt_append: config
                .property_or_default_("storage.encryption.append", "false")
                .unwrap_or(false),
            spam_header: config.value("spam.header.is-spam").and_then(|v| {
                v.split_once(':').map(|(k, v)| {
                    (
                        mail_parser::HeaderName::parse(k.trim().to_string()).unwrap(),
                        v.trim().to_string(),
                    )
                })
            }),
            http_headers: config
                .values("server.http.headers")
                .map(|(_, v)| {
                    if let Some((k, v)) = v.split_once(':') {
                        Ok((
                            hyper::header::HeaderName::from_str(k.trim()).map_err(|err| {
                                format!(
                                    "Invalid header found in property \"server.http.headers\": {}",
                                    err
                                )
                            })?,
                            hyper::header::HeaderValue::from_str(v.trim()).map_err(|err| {
                                format!(
                                    "Invalid header found in property \"server.http.headers\": {}",
                                    err
                                )
                            })?,
                        ))
                    } else {
                        Err(format!(
                            "Invalid header found in property \"server.http.headers\": {}",
                            v
                        ))
                    }
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(|e| config.new_parse_error("server.http.headers", e))
                .unwrap_or_default(),
            push_attempt_interval: config
                .property_or_default_("jmap.push.attempts.interval", "1m")
                .unwrap_or_else(|| Duration::from_secs(60)),
            push_attempts_max: config
                .property_or_default_("jmap.push.attempts.max", "3")
                .unwrap_or(3),
            push_retry_interval: config
                .property_or_default_("jmap.push.retry.interval", "1s")
                .unwrap_or_else(|| Duration::from_secs(1)),
            push_timeout: config
                .property_or_default_("jmap.push.timeout.request", "10s")
                .unwrap_or_else(|| Duration::from_secs(10)),
            push_verify_timeout: config
                .property_or_default_("jmap.push.timeout.verify", "1m")
                .unwrap_or_else(|| Duration::from_secs(60)),
            push_throttle: config
                .property_or_default_("jmap.push.throttle", "1s")
                .unwrap_or_else(|| Duration::from_secs(1)),
            session_purge_frequency: config
                .property_or_default_::<SimpleCron>("jmap.session.purge.frequency", "15 * *")
                .unwrap_or_else(|| SimpleCron::parse_value("15 * *", "").unwrap()),
        };

        // Add capabilities
        jmap.add_capabilites(config);
        jmap
    }
}
