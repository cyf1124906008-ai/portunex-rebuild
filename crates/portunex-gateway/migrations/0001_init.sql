--
-- PostgreSQL database dump
--


-- Dumped from database version 18.4 (Debian 18.4-1.pgdg13+1)
-- Dumped by pg_dump version 18.4 (Debian 18.4-1.pgdg13+1)

--
-- Name: citext; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;
--
-- Name: EXTENSION citext; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';
--
-- Name: fn_get_platform_stats(timestamp with time zone, timestamp with time zone, text, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_get_platform_stats(p_start_time timestamp with time zone, p_end_time timestamp with time zone, p_granularity text DEFAULT 'day'::text, p_user_limit integer DEFAULT 10) RETURNS jsonb
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_trunc  TEXT;
    v_result JSONB;
BEGIN
    v_trunc := CASE p_granularity
        WHEN 'hour'  THEN 'hour'
        WHEN 'week'  THEN 'week'
        WHEN 'month' THEN 'month'
        ELSE 'day'
    END;
    CREATE TEMP TABLE _filtered ON COMMIT DROP AS
        SELECT id, user_id,
               COALESCE(provider, 'unknown') AS provider,
               COALESCE(model, 'unknown')    AS model,
               status,
               input_tokens, output_tokens,
               cache_creation_input_tokens, cache_read_input_tokens,
               started_at, finished_at, first_response_at
        FROM usage_records
        WHERE deleted_at IS NULL
          AND finished_at >= p_start_time
          AND finished_at <= p_end_time;
    ANALYZE _filtered;
    CREATE TEMP TABLE _pts ON COMMIT DROP AS
        SELECT pd.usage_id AS id, SUM(ABS(pd.delta)) AS cost
        FROM points_details pd
        WHERE pd.kind = 'consume' AND pd.deleted_at IS NULL
          AND pd.usage_id IN (SELECT id FROM _filtered)
        GROUP BY pd.usage_id;
    CREATE TEMP TABLE _subs ON COMMIT DROP AS
        SELECT scd.usage_id AS id, SUM(scd.amount) AS cost
        FROM subscription_consume_details scd
        WHERE scd.usage_id IN (SELECT id FROM _filtered)
        GROUP BY scd.usage_id;
    ANALYZE _pts;
    ANALYZE _subs;
    WITH
    enriched AS (
        SELECT f.*,
               COALESCE(p.cost, 0) + COALESCE(s.cost, 0) AS cost
        FROM _filtered f
        LEFT JOIN _pts  p ON p.id = f.id
        LEFT JOIN _subs s ON s.id = f.id
    ),
    stats AS (
        SELECT
            COUNT(*)                                              AS total_requests,
            COUNT(*) FILTER (WHERE status = 'success')           AS successful_requests,
            COUNT(*) FILTER (WHERE status IN ('error','failed')) AS failed_requests,
            COALESCE(SUM(input_tokens), 0)                       AS total_input_tokens,
            COALESCE(SUM(output_tokens), 0)                      AS total_output_tokens,
            COALESCE(SUM(cache_creation_input_tokens), 0)        AS total_cache_creation_tokens,
            COALESCE(SUM(cache_read_input_tokens), 0)            AS total_cache_read_tokens,
            COALESCE(SUM(cost), 0)                               AS total_points_consumed,
            AVG(EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000)
                AS avg_duration_ms,
            AVG(EXTRACT(EPOCH FROM (first_response_at - started_at)) * 1000)
                FILTER (WHERE first_response_at IS NOT NULL)
                AS avg_time_to_first_byte_ms
        FROM enriched
    ),
    model_dist AS (
        SELECT model,
               COUNT(*)                        AS request_count,
               COALESCE(SUM(input_tokens), 0)
                   + COALESCE(SUM(cache_creation_input_tokens), 0)
                   + COALESCE(SUM(cache_read_input_tokens), 0) AS input_tokens,
               COALESCE(SUM(output_tokens), 0) AS output_tokens
        FROM enriched
        GROUP BY model
        ORDER BY request_count DESC
    ),
    provider_dist AS (
        SELECT provider,
               COUNT(*)                        AS request_count,
               COALESCE(SUM(input_tokens), 0)
                   + COALESCE(SUM(cache_creation_input_tokens), 0)
                   + COALESCE(SUM(cache_read_input_tokens), 0) AS input_tokens,
               COALESCE(SUM(output_tokens), 0) AS output_tokens,
               COALESCE(SUM(cost), 0)          AS points_consumed
        FROM enriched
        GROUP BY provider
        ORDER BY request_count DESC
    ),
    user_dist AS (
        SELECT e.user_id,
               u.email,
               COUNT(*)                        AS request_count,
               COALESCE(SUM(e.input_tokens), 0)
                   + COALESCE(SUM(e.cache_creation_input_tokens), 0)
                   + COALESCE(SUM(e.cache_read_input_tokens), 0) AS input_tokens,
               COALESCE(SUM(e.output_tokens), 0) AS output_tokens,
               COALESCE(SUM(e.cost), 0)          AS points_consumed
        FROM enriched e
        LEFT JOIN users u ON e.user_id = u.id
        GROUP BY e.user_id, u.email
        ORDER BY request_count DESC
        LIMIT p_user_limit
    ),
    time_series AS (
        SELECT date_trunc(v_trunc, finished_at) AS time_bucket,
               COUNT(*)                          AS request_count,
               COUNT(*) FILTER (WHERE status = 'success') AS successful_requests,
               COALESCE(SUM(input_tokens), 0)
                   + COALESCE(SUM(cache_creation_input_tokens), 0)
                   + COALESCE(SUM(cache_read_input_tokens), 0) AS input_tokens,
               COALESCE(SUM(output_tokens), 0)   AS output_tokens,
               COALESCE(SUM(cost), 0)            AS points_consumed
        FROM enriched
        GROUP BY time_bucket
        ORDER BY time_bucket
    )
    SELECT jsonb_build_object(
        'total_requests',              s.total_requests,
        'successful_requests',         s.successful_requests,
        'failed_requests',             s.failed_requests,
        'total_input_tokens',          s.total_input_tokens,
        'total_output_tokens',         s.total_output_tokens,
        'total_cache_creation_tokens', s.total_cache_creation_tokens,
        'total_cache_read_tokens',     s.total_cache_read_tokens,
        'total_points_consumed',       s.total_points_consumed,
        'avg_duration_ms',             s.avg_duration_ms,
        'avg_time_to_first_byte_ms',   s.avg_time_to_first_byte_ms,

        'model_distribution', COALESCE((
            SELECT jsonb_agg(jsonb_build_object(
                'model',         md.model,
                'request_count', md.request_count,
                'input_tokens',  md.input_tokens,
                'output_tokens', md.output_tokens
            ) ORDER BY md.request_count DESC)
            FROM model_dist md
        ), '[]'::jsonb),

        'provider_distribution', COALESCE((
            SELECT jsonb_agg(jsonb_build_object(
                'provider',        pd.provider,
                'request_count',   pd.request_count,
                'input_tokens',    pd.input_tokens,
                'output_tokens',   pd.output_tokens,
                'points_consumed', pd.points_consumed
            ) ORDER BY pd.request_count DESC)
            FROM provider_dist pd
        ), '[]'::jsonb),

        'user_distribution', COALESCE((
            SELECT jsonb_agg(jsonb_build_object(
                'user_id',         ud.user_id,
                'email',           ud.email,
                'request_count',   ud.request_count,
                'input_tokens',    ud.input_tokens,
                'output_tokens',   ud.output_tokens,
                'points_consumed', ud.points_consumed
            ) ORDER BY ud.request_count DESC)
            FROM user_dist ud
        ), '[]'::jsonb),

        'time_series', COALESCE((
            SELECT jsonb_agg(jsonb_build_object(
                'time_bucket',         t.time_bucket,
                'request_count',       t.request_count,
                'successful_requests', t.successful_requests,
                'input_tokens',        t.input_tokens,
                'output_tokens',       t.output_tokens,
                'points_consumed',     t.points_consumed
            ) ORDER BY t.time_bucket)
            FROM time_series t
        ), '[]'::jsonb)
    )
    INTO v_result
    FROM stats s;
    RETURN v_result;
END;
$$;
--
-- Name: api_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.api_keys (
    id bigint NOT NULL,
    user_id bigint,
    key_text text,
    prefix text,
    active boolean DEFAULT true,
    settings jsonb,
    created_at timestamp with time zone DEFAULT now(),
    rotated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    name text,
    last_used_at timestamp with time zone
);
--
-- Name: auth_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.auth_sessions (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    token text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone DEFAULT now(),
    last_used_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: kv_store; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.kv_store (
    id bigint NOT NULL,
    key text NOT NULL,
    value text,
    category text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    expires_at timestamp with time zone
);
--
-- Name: magic_link_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.magic_link_tokens (
    id bigint NOT NULL,
    email public.citext NOT NULL,
    token text NOT NULL,
    user_id bigint,
    is_new_user boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now(),
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    ip_address text,
    user_agent text
);
--
-- Name: model_alias_pricing; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.model_alias_pricing (
    id bigint NOT NULL,
    model_alias text,
    input_per_1k numeric(30,18),
    output_per_1k numeric(30,18),
    cache_create_per_1k numeric(30,18),
    cache_read_per_1k numeric(30,18),
    cache_1h_per_1k numeric(30,18),
    effective_from timestamp with time zone,
    effective_to timestamp with time zone,
    priority integer
);
--
-- Name: model_aliases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.model_aliases (
    id bigint NOT NULL,
    alias text NOT NULL,
    kind text NOT NULL,
    upstream_model text,
    priority integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
--
-- Name: oauth_identities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_identities (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    provider text NOT NULL,
    provider_user_id text NOT NULL,
    provider_username text,
    profile jsonb DEFAULT '{}'::jsonb,
    access_token text,
    refresh_token text,
    token_expires_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: oauth_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_states (
    id bigint NOT NULL,
    state text,
    code_verifier text,
    created_at timestamp with time zone DEFAULT now(),
    expires_at timestamp with time zone,
    deleted_at timestamp with time zone,
    purpose text DEFAULT 'provider'::text,
    binding_user_id bigint
);
--
-- Name: oidc_authorization_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oidc_authorization_codes (
    id bigint NOT NULL,
    code text NOT NULL,
    client_id text NOT NULL,
    user_id bigint NOT NULL,
    redirect_uri text NOT NULL,
    scopes text[] NOT NULL,
    nonce text,
    code_challenge text,
    code_challenge_method text,
    expires_at timestamp with time zone NOT NULL,
    consumed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now()
);
--
-- Name: oidc_clients; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oidc_clients (
    id bigint NOT NULL,
    client_id text NOT NULL,
    client_secret_hash text,
    client_name text NOT NULL,
    redirect_uris text[] NOT NULL,
    allowed_scopes text[] DEFAULT '{openid}'::text[] NOT NULL,
    grant_types text[] DEFAULT '{authorization_code}'::text[] NOT NULL,
    token_endpoint_auth_method text DEFAULT 'client_secret_basic'::text NOT NULL,
    access_token_ttl_secs integer DEFAULT 3600,
    refresh_token_ttl_secs integer DEFAULT 86400,
    active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: oidc_consents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oidc_consents (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    client_id text NOT NULL,
    scopes text[] NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    revoked_at timestamp with time zone
);
--
-- Name: oidc_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oidc_tokens (
    id bigint NOT NULL,
    token_hash text NOT NULL,
    token_type text NOT NULL,
    client_id text NOT NULL,
    user_id bigint,
    scopes text[] NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    revoked_at timestamp with time zone,
    parent_id bigint,
    created_at timestamp with time zone DEFAULT now()
);
--
-- Name: orders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.orders (
    id bigint NOT NULL,
    order_no text,
    user_id bigint,
    channel_id bigint,
    product_type text,
    product_info jsonb DEFAULT '{}'::jsonb,
    amount numeric(30,18),
    amount_cents bigint,
    status text DEFAULT 'pending'::text,
    payment_url text,
    payment_meta jsonb DEFAULT '{}'::jsonb,
    settled boolean DEFAULT false,
    settled_at timestamp with time zone,
    expires_at timestamp with time zone,
    paid_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: payment_channels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payment_channels (
    id bigint NOT NULL,
    code text,
    name text,
    provider_type text,
    config_json jsonb DEFAULT '{}'::jsonb,
    priority integer DEFAULT 0,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: points_details; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.points_details (
    id bigint NOT NULL,
    user_id bigint,
    usage_id bigint,
    kind text,
    delta numeric(30,18) NOT NULL,
    balance_after numeric(30,18),
    reason text,
    pricing_snapshot jsonb,
    created_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: COLUMN points_details.pricing_snapshot; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.points_details.pricing_snapshot IS 'Price snapshot at the time of charge (for consume records)';
--
-- Name: provider_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.provider_credentials (
    id bigint NOT NULL,
    provider_id bigint,
    auth_type text,
    secret text,
    meta_json jsonb DEFAULT '{}'::jsonb,
    deleted_at timestamp with time zone
);
--
-- Name: provider_model_pricing; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.provider_model_pricing (
    id bigint NOT NULL,
    provider_id bigint,
    model_id text,
    input_per_1k numeric(30,18),
    output_per_1k numeric(30,18),
    cache_create_per_1k numeric(30,18),
    cache_read_per_1k numeric(30,18),
    cache_1h_per_1k numeric(30,18),
    effective_from timestamp with time zone,
    effective_to timestamp with time zone,
    priority integer
);
--
-- Name: provider_type_pricing; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.provider_type_pricing (
    id bigint NOT NULL,
    kind text NOT NULL,
    auth_type text,
    model_id text,
    input_per_1k numeric(30,18),
    output_per_1k numeric(30,18),
    cache_create_per_1k numeric(30,18),
    cache_read_per_1k numeric(30,18),
    cache_1h_per_1k numeric(30,18),
    multiplier numeric(10,6),
    effective_from timestamp with time zone,
    effective_to timestamp with time zone,
    priority integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);
--
-- Name: COLUMN provider_type_pricing.multiplier; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.provider_type_pricing.multiplier IS 'Price multiplier applied to model_alias_pricing. When set, individual price fields are ignored.';
--
-- Name: provider_window_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.provider_window_configs (
    id bigint NOT NULL,
    provider_kind text,
    window_type text,
    window_seconds bigint,
    stat_type text,
    limit_value bigint,
    model_pattern text,
    priority integer DEFAULT 0,
    enabled boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: provider_window_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.provider_window_states (
    id bigint NOT NULL,
    config_id bigint,
    provider_id bigint,
    window_start timestamp with time zone,
    window_end timestamp with time zone,
    input_tokens bigint DEFAULT 0,
    output_tokens bigint DEFAULT 0,
    cache_creation_input_tokens bigint DEFAULT 0,
    cache_read_input_tokens bigint DEFAULT 0,
    request_count bigint DEFAULT 0,
    last_request_at timestamp with time zone,
    utilization double precision,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
--
-- Name: COLUMN provider_window_states.utilization; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.provider_window_states.utilization IS 'Upstream rate limit utilization percentage (0.0 - 1.0) from response headers';
--
-- Name: providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.providers (
    id bigint NOT NULL,
    kind text,
    name text,
    base_url text,
    http_proxy text,
    socks5_proxy text,
    weight integer DEFAULT 1,
    healthy boolean DEFAULT true,
    last_error jsonb,
    last_checked_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone,
    rpm_limit integer,
    rps_limit integer,
    max_concurrent integer,
    available_from text,
    available_until text,
    cooldown_until timestamp with time zone
);
--
-- Name: COLUMN providers.rpm_limit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.providers.rpm_limit IS 'RPM limit for this provider. NULL means no limit.';
--
-- Name: COLUMN providers.rps_limit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.providers.rps_limit IS 'RPS limit for this provider. NULL means no limit.';
--
-- Name: COLUMN providers.available_from; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.providers.available_from IS 'Start time of availability window in HH:MM format (24-hour)';
--
-- Name: COLUMN providers.available_until; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.providers.available_until IS 'End time of availability window in HH:MM format (24-hour)';
--
-- Name: redemption_attempts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.redemption_attempts (
    id bigint NOT NULL,
    ip_address text NOT NULL,
    attempted_code text,
    success boolean DEFAULT false NOT NULL,
    attempted_at timestamp with time zone DEFAULT now() NOT NULL
);
--
-- Name: redemption_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.redemption_codes (
    id bigint NOT NULL,
    code text NOT NULL,
    name text,
    reward_type text NOT NULL,
    reward_value jsonb NOT NULL,
    max_uses integer,
    used_count integer DEFAULT 0 NOT NULL,
    valid_from timestamp with time zone,
    valid_until timestamp with time zone,
    created_by bigint,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);
--
-- Name: redemption_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.redemption_records (
    id bigint NOT NULL,
    code_id bigint NOT NULL,
    user_id bigint NOT NULL,
    reward_type text NOT NULL,
    reward_snapshot jsonb,
    result jsonb,
    ip_address text,
    user_agent text,
    redeemed_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);
--
-- Name: sticky_routing; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sticky_routing (
    routing_key text NOT NULL,
    model_alias text NOT NULL,
    provider_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    last_used_at timestamp with time zone DEFAULT now(),
    expires_at timestamp with time zone
);
--
-- Name: subscription_consume_details; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subscription_consume_details (
    id bigint NOT NULL,
    subscription_id bigint NOT NULL,
    usage_id bigint,
    amount numeric(30,18) NOT NULL,
    reason text,
    daily_state_id bigint,
    weekly_state_id bigint,
    monthly_state_id bigint,
    created_at timestamp with time zone DEFAULT now(),
    pricing_snapshot jsonb
);
--
-- Name: COLUMN subscription_consume_details.pricing_snapshot; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.subscription_consume_details.pricing_snapshot IS 'Price snapshot at the time of charge';
--
-- Name: subscription_plans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subscription_plans (
    id bigint NOT NULL,
    name text NOT NULL,
    code text NOT NULL,
    description text,
    is_special boolean DEFAULT false,
    level integer DEFAULT 0,
    duration_days integer NOT NULL,
    daily_limit numeric(30,18),
    weekly_limit numeric(30,18),
    monthly_limit numeric(30,18),
    price numeric(30,18) NOT NULL,
    price_cents bigint NOT NULL,
    active boolean DEFAULT true,
    sort_order integer DEFAULT 0,
    meta_json jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: subscription_usage_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subscription_usage_states (
    id bigint NOT NULL,
    subscription_id bigint NOT NULL,
    window_type text NOT NULL,
    window_start timestamp with time zone NOT NULL,
    window_end timestamp with time zone NOT NULL,
    used_amount numeric(30,18) DEFAULT 0,
    limit_amount numeric(30,18),
    last_used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
--
-- Name: usage_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.usage_records (
    id bigint NOT NULL,
    user_id bigint,
    api_key_id bigint,
    provider_id bigint,
    provider text,
    facade text,
    model text,
    mode text,
    request_id_upstream text,
    trace_id text,
    started_at timestamp with time zone,
    first_response_at timestamp with time zone,
    first_output_at timestamp with time zone,
    finished_at timestamp with time zone,
    status text,
    upstream_status text,
    input_tokens bigint,
    output_tokens bigint,
    cache_creation_input_tokens bigint,
    cache_read_input_tokens bigint,
    request_bytes bigint,
    response_bytes bigint,
    error_code text,
    error_message text,
    deleted_at timestamp with time zone
);
--
-- Name: COLUMN usage_records.first_response_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.usage_records.first_response_at IS 'Timestamp of first response from upstream - TTFB (streaming only)';
--
-- Name: COLUMN usage_records.first_output_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.usage_records.first_output_at IS 'Timestamp of first content output from upstream (streaming only)';
--
-- Name: user_subscriptions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_subscriptions (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    plan_id bigint NOT NULL,
    order_id bigint,
    is_special boolean NOT NULL,
    level integer NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    started_at timestamp with time zone NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    daily_limit numeric(30,18),
    weekly_limit numeric(30,18),
    monthly_limit numeric(30,18),
    meta_json jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);
--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    email public.citext,
    password_phc text,
    points numeric(30,18) DEFAULT 0,
    role text DEFAULT 'user'::text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone,
    can_purchase_subscription boolean DEFAULT false NOT NULL,
    daily_recharge_limit numeric(30,18) DEFAULT 0 NOT NULL,
    CONSTRAINT users_role_check CHECK ((role = ANY (ARRAY['admin'::text, 'user'::text])))
);
--
-- Name: COLUMN users.can_purchase_subscription; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.can_purchase_subscription IS 'Whether the user is allowed to purchase subscription plans (default disabled)';
--
-- Name: COLUMN users.daily_recharge_limit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.daily_recharge_limit IS 'Maximum total points the user can recharge per day (UTC+8 boundary, 0 = recharge disabled)';
--
-- Name: vendor_models; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vendor_models (
    id bigint NOT NULL,
    kind text,
    model_id text,
    active boolean,
    meta_json jsonb DEFAULT '{}'::jsonb
);
--
-- Name: api_keys api_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_pkey PRIMARY KEY (id);
--
-- Name: auth_sessions auth_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.auth_sessions
    ADD CONSTRAINT auth_sessions_pkey PRIMARY KEY (id);
--
-- Name: kv_store kv_store_category_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kv_store
    ADD CONSTRAINT kv_store_category_key_key UNIQUE (category, key);
--
-- Name: kv_store kv_store_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kv_store
    ADD CONSTRAINT kv_store_pkey PRIMARY KEY (id);
--
-- Name: magic_link_tokens magic_link_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.magic_link_tokens
    ADD CONSTRAINT magic_link_tokens_pkey PRIMARY KEY (id);
--
-- Name: magic_link_tokens magic_link_tokens_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.magic_link_tokens
    ADD CONSTRAINT magic_link_tokens_token_key UNIQUE (token);
--
-- Name: model_alias_pricing model_alias_pricing_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model_alias_pricing
    ADD CONSTRAINT model_alias_pricing_pkey PRIMARY KEY (id);
--
-- Name: model_aliases model_aliases_alias_kind_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model_aliases
    ADD CONSTRAINT model_aliases_alias_kind_key UNIQUE (alias, kind);
--
-- Name: model_aliases model_aliases_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model_aliases
    ADD CONSTRAINT model_aliases_pkey PRIMARY KEY (id);
--
-- Name: oauth_identities oauth_identities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_identities
    ADD CONSTRAINT oauth_identities_pkey PRIMARY KEY (id);
--
-- Name: oauth_states oauth_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_states
    ADD CONSTRAINT oauth_states_pkey PRIMARY KEY (id);
--
-- Name: oidc_authorization_codes oidc_authorization_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_authorization_codes
    ADD CONSTRAINT oidc_authorization_codes_pkey PRIMARY KEY (id);
--
-- Name: oidc_clients oidc_clients_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_clients
    ADD CONSTRAINT oidc_clients_pkey PRIMARY KEY (id);
--
-- Name: oidc_consents oidc_consents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_consents
    ADD CONSTRAINT oidc_consents_pkey PRIMARY KEY (id);
--
-- Name: oidc_tokens oidc_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_tokens
    ADD CONSTRAINT oidc_tokens_pkey PRIMARY KEY (id);
--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);
--
-- Name: payment_channels payment_channels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_channels
    ADD CONSTRAINT payment_channels_pkey PRIMARY KEY (id);
--
-- Name: points_details points_details_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.points_details
    ADD CONSTRAINT points_details_pkey PRIMARY KEY (id);
--
-- Name: provider_credentials provider_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_credentials
    ADD CONSTRAINT provider_credentials_pkey PRIMARY KEY (id);
--
-- Name: provider_model_pricing provider_model_pricing_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_model_pricing
    ADD CONSTRAINT provider_model_pricing_pkey PRIMARY KEY (id);
--
-- Name: provider_type_pricing provider_type_pricing_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_type_pricing
    ADD CONSTRAINT provider_type_pricing_pkey PRIMARY KEY (id);
--
-- Name: provider_window_configs provider_window_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_window_configs
    ADD CONSTRAINT provider_window_configs_pkey PRIMARY KEY (id);
--
-- Name: provider_window_states provider_window_states_config_id_provider_id_window_start_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_window_states
    ADD CONSTRAINT provider_window_states_config_id_provider_id_window_start_key UNIQUE (config_id, provider_id, window_start);
--
-- Name: provider_window_states provider_window_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_window_states
    ADD CONSTRAINT provider_window_states_pkey PRIMARY KEY (id);
--
-- Name: providers providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.providers
    ADD CONSTRAINT providers_pkey PRIMARY KEY (id);
--
-- Name: redemption_attempts redemption_attempts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_attempts
    ADD CONSTRAINT redemption_attempts_pkey PRIMARY KEY (id);
--
-- Name: redemption_codes redemption_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_codes
    ADD CONSTRAINT redemption_codes_pkey PRIMARY KEY (id);
--
-- Name: redemption_records redemption_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_records
    ADD CONSTRAINT redemption_records_pkey PRIMARY KEY (id);
--
-- Name: sticky_routing sticky_routing_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sticky_routing
    ADD CONSTRAINT sticky_routing_pkey PRIMARY KEY (routing_key, model_alias);
--
-- Name: subscription_consume_details subscription_consume_details_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_pkey PRIMARY KEY (id);
--
-- Name: subscription_plans subscription_plans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_plans
    ADD CONSTRAINT subscription_plans_pkey PRIMARY KEY (id);
--
-- Name: subscription_usage_states subscription_usage_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_usage_states
    ADD CONSTRAINT subscription_usage_states_pkey PRIMARY KEY (id);
--
-- Name: usage_records usage_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_records
    ADD CONSTRAINT usage_records_pkey PRIMARY KEY (id);
--
-- Name: user_subscriptions user_subscriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_subscriptions
    ADD CONSTRAINT user_subscriptions_pkey PRIMARY KEY (id);
--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);
--
-- Name: vendor_models vendor_models_kind_model_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vendor_models
    ADD CONSTRAINT vendor_models_kind_model_id_key UNIQUE (kind, model_id);
--
-- Name: vendor_models vendor_models_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vendor_models
    ADD CONSTRAINT vendor_models_pkey PRIMARY KEY (id);
--
-- Name: idx_alias_pricing_alias; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_alias_pricing_alias ON public.model_alias_pricing USING btree (model_alias);
--
-- Name: idx_api_keys_created_at_desc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_created_at_desc ON public.api_keys USING btree (created_at DESC) WHERE (deleted_at IS NULL);
--
-- Name: idx_api_keys_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_deleted_at ON public.api_keys USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_api_keys_key_text_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_api_keys_key_text_unique ON public.api_keys USING btree (key_text) WHERE (deleted_at IS NULL);
--
-- Name: idx_api_keys_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_last_used ON public.api_keys USING btree (last_used_at DESC NULLS LAST) WHERE (deleted_at IS NULL);
--
-- Name: idx_api_keys_prefix_text; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_prefix_text ON public.api_keys USING btree (prefix, key_text) WHERE (deleted_at IS NULL);
--
-- Name: idx_api_keys_settings; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_settings ON public.api_keys USING gin (settings);
--
-- Name: idx_auth_sessions_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_deleted_at ON public.auth_sessions USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_auth_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_expires ON public.auth_sessions USING btree (expires_at);
--
-- Name: idx_auth_sessions_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_token ON public.auth_sessions USING btree (token);
--
-- Name: idx_auth_sessions_token_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_auth_sessions_token_unique ON public.auth_sessions USING btree (token) WHERE (deleted_at IS NULL);
--
-- Name: idx_auth_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_user ON public.auth_sessions USING btree (user_id);
--
-- Name: idx_kv_store_category_key; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_kv_store_category_key ON public.kv_store USING hash ((((category || ':'::text) || key)));
--
-- Name: idx_kv_store_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_kv_store_expires ON public.kv_store USING btree (expires_at);
--
-- Name: idx_magic_link_tokens_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_magic_link_tokens_email ON public.magic_link_tokens USING btree (email);
--
-- Name: idx_magic_link_tokens_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_magic_link_tokens_expires ON public.magic_link_tokens USING btree (expires_at);
--
-- Name: idx_magic_link_tokens_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_magic_link_tokens_token ON public.magic_link_tokens USING btree (token);
--
-- Name: idx_model_aliases_alias; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_model_aliases_alias ON public.model_aliases USING btree (alias);
--
-- Name: idx_model_aliases_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_model_aliases_kind ON public.model_aliases USING btree (kind);
--
-- Name: idx_oauth_identities_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_identities_provider ON public.oauth_identities USING btree (provider) WHERE (deleted_at IS NULL);
--
-- Name: idx_oauth_identities_provider_user_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oauth_identities_provider_user_unique ON public.oauth_identities USING btree (provider, provider_user_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_oauth_identities_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_identities_user ON public.oauth_identities USING btree (user_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_oauth_states_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_states_deleted_at ON public.oauth_states USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_oauth_states_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_states_expires ON public.oauth_states USING btree (expires_at);
--
-- Name: idx_oauth_states_state; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_states_state ON public.oauth_states USING btree (state);
--
-- Name: idx_oauth_states_state_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oauth_states_state_unique ON public.oauth_states USING btree (state) WHERE (deleted_at IS NULL);
--
-- Name: idx_oidc_auth_codes_code_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oidc_auth_codes_code_unique ON public.oidc_authorization_codes USING btree (code) WHERE (consumed_at IS NULL);
--
-- Name: idx_oidc_auth_codes_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oidc_auth_codes_expires ON public.oidc_authorization_codes USING btree (expires_at);
--
-- Name: idx_oidc_clients_client_id_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oidc_clients_client_id_unique ON public.oidc_clients USING btree (client_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_oidc_consents_user_client; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oidc_consents_user_client ON public.oidc_consents USING btree (user_id, client_id) WHERE (revoked_at IS NULL);
--
-- Name: idx_oidc_tokens_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oidc_tokens_expires ON public.oidc_tokens USING btree (expires_at);
--
-- Name: idx_oidc_tokens_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_oidc_tokens_hash ON public.oidc_tokens USING btree (token_hash);
--
-- Name: idx_oidc_tokens_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oidc_tokens_parent ON public.oidc_tokens USING btree (parent_id);
--
-- Name: idx_oidc_tokens_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oidc_tokens_user ON public.oidc_tokens USING btree (user_id);
--
-- Name: idx_orders_order_no; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_order_no ON public.orders USING btree (order_no) WHERE (deleted_at IS NULL);
--
-- Name: idx_orders_order_no_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_orders_order_no_unique ON public.orders USING btree (order_no) WHERE (deleted_at IS NULL);
--
-- Name: idx_orders_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_pending ON public.orders USING btree (status, expires_at) WHERE ((status = ANY (ARRAY['pending'::text, 'paying'::text])) AND (deleted_at IS NULL));
--
-- Name: idx_orders_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_status ON public.orders USING btree (status) WHERE (deleted_at IS NULL);
--
-- Name: idx_orders_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_orders_user ON public.orders USING btree (user_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_payment_channels_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payment_channels_active ON public.payment_channels USING btree (active) WHERE (deleted_at IS NULL);
--
-- Name: idx_payment_channels_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payment_channels_code ON public.payment_channels USING btree (code) WHERE (deleted_at IS NULL);
--
-- Name: idx_payment_channels_code_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_payment_channels_code_unique ON public.payment_channels USING btree (code) WHERE (deleted_at IS NULL);
--
-- Name: idx_points_consume_covering; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_points_consume_covering ON public.points_details USING btree (usage_id) INCLUDE (delta) WHERE ((kind = 'consume'::text) AND (deleted_at IS NULL));
--
-- Name: idx_points_details_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_points_details_deleted_at ON public.points_details USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_points_details_usage_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_points_details_usage_id ON public.points_details USING btree (usage_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_points_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_points_user_time ON public.points_details USING btree (user_id, created_at);
--
-- Name: idx_provider_credentials_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_provider_credentials_deleted_at ON public.provider_credentials USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_provider_credentials_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_provider_credentials_provider ON public.provider_credentials USING btree (provider_id);
--
-- Name: idx_provider_model_pricing; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_provider_model_pricing ON public.provider_model_pricing USING btree (provider_id, model_id);
--
-- Name: idx_provider_type_pricing_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_provider_type_pricing_lookup ON public.provider_type_pricing USING btree (kind, auth_type, model_id);
--
-- Name: idx_provider_type_pricing_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_provider_type_pricing_time ON public.provider_type_pricing USING btree (effective_from, effective_to) WHERE (deleted_at IS NULL);
--
-- Name: idx_provider_type_pricing_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_provider_type_pricing_unique ON public.provider_type_pricing USING btree (kind, COALESCE(auth_type, ''::text), COALESCE(model_id, ''::text)) WHERE (deleted_at IS NULL);
--
-- Name: idx_providers_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_providers_deleted_at ON public.providers USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_redemption_attempts_ip_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_attempts_ip_time ON public.redemption_attempts USING btree (ip_address, attempted_at);
--
-- Name: idx_redemption_codes_code_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_redemption_codes_code_unique ON public.redemption_codes USING btree (code) WHERE (deleted_at IS NULL);
--
-- Name: idx_redemption_codes_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_codes_deleted_at ON public.redemption_codes USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_redemption_codes_valid_range; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_codes_valid_range ON public.redemption_codes USING btree (valid_from, valid_until) WHERE (deleted_at IS NULL);
--
-- Name: idx_redemption_records_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_records_code ON public.redemption_records USING btree (code_id);
--
-- Name: idx_redemption_records_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_records_time ON public.redemption_records USING btree (redeemed_at);
--
-- Name: idx_redemption_records_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_redemption_records_user ON public.redemption_records USING btree (user_id);
--
-- Name: idx_redemption_records_user_code_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_redemption_records_user_code_unique ON public.redemption_records USING btree (user_id, code_id);
--
-- Name: idx_sticky_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sticky_provider ON public.sticky_routing USING btree (provider_id);
--
-- Name: idx_subscription_consume_subscription; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subscription_consume_subscription ON public.subscription_consume_details USING btree (subscription_id, created_at);
--
-- Name: idx_subscription_consume_usage; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subscription_consume_usage ON public.subscription_consume_details USING btree (usage_id);
--
-- Name: idx_subscription_plans_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subscription_plans_active ON public.subscription_plans USING btree (active, is_special, level) WHERE (deleted_at IS NULL);
--
-- Name: idx_subscription_plans_code_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_subscription_plans_code_unique ON public.subscription_plans USING btree (code) WHERE (deleted_at IS NULL);
--
-- Name: idx_subscription_usage_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_subscription_usage_unique ON public.subscription_usage_states USING btree (subscription_id, window_type, window_start);
--
-- Name: idx_subscription_usage_window; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subscription_usage_window ON public.subscription_usage_states USING btree (subscription_id, window_end);
--
-- Name: idx_usage_api_key_stats_covering; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_api_key_stats_covering ON public.usage_records USING btree (api_key_id) INCLUDE (id, status, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, started_at, finished_at, first_response_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_provider_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_provider_id ON public.usage_records USING btree (provider_id);
--
-- Name: idx_usage_records_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_records_deleted_at ON public.usage_records USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_records_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_records_trace_id ON public.usage_records USING btree (trace_id);
--
-- Name: idx_usage_stats_covering; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_stats_covering ON public.usage_records USING btree (finished_at) INCLUDE (id, user_id, provider, model, status, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, started_at, first_response_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_user_facade_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_user_facade_time ON public.usage_records USING btree (user_id, facade, finished_at DESC) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_user_model_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_user_model_time ON public.usage_records USING btree (user_id, model, finished_at DESC) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_user_provider_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_user_provider_time ON public.usage_records USING btree (user_id, provider, finished_at DESC) WHERE (deleted_at IS NULL);
--
-- Name: idx_usage_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_user_time ON public.usage_records USING btree (user_id, finished_at);
--
-- Name: idx_usage_user_time_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_user_time_status ON public.usage_records USING btree (user_id, finished_at DESC, status) WHERE (deleted_at IS NULL);
--
-- Name: idx_user_subscriptions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_subscriptions_expires ON public.user_subscriptions USING btree (expires_at) WHERE ((deleted_at IS NULL) AND (status = 'active'::text));
--
-- Name: idx_user_subscriptions_normal_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_user_subscriptions_normal_unique ON public.user_subscriptions USING btree (user_id) WHERE ((deleted_at IS NULL) AND (status = 'active'::text) AND (is_special = false));
--
-- Name: idx_user_subscriptions_order; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_subscriptions_order ON public.user_subscriptions USING btree (order_id) WHERE (deleted_at IS NULL);
--
-- Name: idx_user_subscriptions_user_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_subscriptions_user_active ON public.user_subscriptions USING btree (user_id, status, is_special, expires_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_users_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_deleted_at ON public.users USING btree (deleted_at) WHERE (deleted_at IS NULL);
--
-- Name: idx_users_email_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_users_email_unique ON public.users USING btree (email) WHERE (deleted_at IS NULL);
--
-- Name: idx_window_configs_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_window_configs_kind ON public.provider_window_configs USING btree (provider_kind) WHERE ((deleted_at IS NULL) AND (enabled = true));
--
-- Name: idx_window_configs_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_window_configs_unique ON public.provider_window_configs USING btree (provider_kind, window_type, stat_type, COALESCE(model_pattern, ''::text)) WHERE (deleted_at IS NULL);
--
-- Name: idx_window_states_config; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_window_states_config ON public.provider_window_states USING btree (config_id, window_start);
--
-- Name: idx_window_states_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_window_states_provider ON public.provider_window_states USING btree (provider_id, window_end);
--
-- Name: api_keys api_keys_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: auth_sessions auth_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.auth_sessions
    ADD CONSTRAINT auth_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: magic_link_tokens magic_link_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.magic_link_tokens
    ADD CONSTRAINT magic_link_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);
--
-- Name: oauth_identities oauth_identities_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_identities
    ADD CONSTRAINT oauth_identities_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: oidc_authorization_codes oidc_authorization_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_authorization_codes
    ADD CONSTRAINT oidc_authorization_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);
--
-- Name: oidc_consents oidc_consents_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oidc_consents
    ADD CONSTRAINT oidc_consents_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);
--
-- Name: orders orders_channel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_channel_id_fkey FOREIGN KEY (channel_id) REFERENCES public.payment_channels(id);
--
-- Name: orders orders_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: points_details points_details_usage_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.points_details
    ADD CONSTRAINT points_details_usage_id_fkey FOREIGN KEY (usage_id) REFERENCES public.usage_records(id) ON DELETE SET NULL;
--
-- Name: points_details points_details_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.points_details
    ADD CONSTRAINT points_details_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: provider_credentials provider_credentials_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_credentials
    ADD CONSTRAINT provider_credentials_provider_id_fkey FOREIGN KEY (provider_id) REFERENCES public.providers(id) ON DELETE CASCADE;
--
-- Name: provider_model_pricing provider_model_pricing_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_model_pricing
    ADD CONSTRAINT provider_model_pricing_provider_id_fkey FOREIGN KEY (provider_id) REFERENCES public.providers(id) ON DELETE CASCADE;
--
-- Name: provider_window_states provider_window_states_config_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_window_states
    ADD CONSTRAINT provider_window_states_config_id_fkey FOREIGN KEY (config_id) REFERENCES public.provider_window_configs(id) ON DELETE CASCADE;
--
-- Name: provider_window_states provider_window_states_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.provider_window_states
    ADD CONSTRAINT provider_window_states_provider_id_fkey FOREIGN KEY (provider_id) REFERENCES public.providers(id) ON DELETE CASCADE;
--
-- Name: redemption_codes redemption_codes_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_codes
    ADD CONSTRAINT redemption_codes_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);
--
-- Name: redemption_records redemption_records_code_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_records
    ADD CONSTRAINT redemption_records_code_id_fkey FOREIGN KEY (code_id) REFERENCES public.redemption_codes(id);
--
-- Name: redemption_records redemption_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.redemption_records
    ADD CONSTRAINT redemption_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);
--
-- Name: sticky_routing sticky_routing_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sticky_routing
    ADD CONSTRAINT sticky_routing_provider_id_fkey FOREIGN KEY (provider_id) REFERENCES public.providers(id) ON DELETE RESTRICT;
--
-- Name: subscription_consume_details subscription_consume_details_daily_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_daily_state_id_fkey FOREIGN KEY (daily_state_id) REFERENCES public.subscription_usage_states(id) ON DELETE SET NULL;
--
-- Name: subscription_consume_details subscription_consume_details_monthly_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_monthly_state_id_fkey FOREIGN KEY (monthly_state_id) REFERENCES public.subscription_usage_states(id) ON DELETE SET NULL;
--
-- Name: subscription_consume_details subscription_consume_details_subscription_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_subscription_id_fkey FOREIGN KEY (subscription_id) REFERENCES public.user_subscriptions(id) ON DELETE CASCADE;
--
-- Name: subscription_consume_details subscription_consume_details_usage_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_usage_id_fkey FOREIGN KEY (usage_id) REFERENCES public.usage_records(id) ON DELETE SET NULL;
--
-- Name: subscription_consume_details subscription_consume_details_weekly_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_consume_details
    ADD CONSTRAINT subscription_consume_details_weekly_state_id_fkey FOREIGN KEY (weekly_state_id) REFERENCES public.subscription_usage_states(id) ON DELETE SET NULL;
--
-- Name: subscription_usage_states subscription_usage_states_subscription_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_usage_states
    ADD CONSTRAINT subscription_usage_states_subscription_id_fkey FOREIGN KEY (subscription_id) REFERENCES public.user_subscriptions(id) ON DELETE CASCADE;
--
-- Name: usage_records usage_records_api_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_records
    ADD CONSTRAINT usage_records_api_key_id_fkey FOREIGN KEY (api_key_id) REFERENCES public.api_keys(id) ON DELETE SET NULL;
--
-- Name: usage_records usage_records_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_records
    ADD CONSTRAINT usage_records_provider_id_fkey FOREIGN KEY (provider_id) REFERENCES public.providers(id);
--
-- Name: usage_records usage_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_records
    ADD CONSTRAINT usage_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: user_subscriptions user_subscriptions_order_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_subscriptions
    ADD CONSTRAINT user_subscriptions_order_id_fkey FOREIGN KEY (order_id) REFERENCES public.orders(id);
--
-- Name: user_subscriptions user_subscriptions_plan_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_subscriptions
    ADD CONSTRAINT user_subscriptions_plan_id_fkey FOREIGN KEY (plan_id) REFERENCES public.subscription_plans(id);
--
-- Name: user_subscriptions user_subscriptions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_subscriptions
    ADD CONSTRAINT user_subscriptions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);
--
-- PostgreSQL database dump complete
--


