use fpx_lib::{
    api::models::{AttributeValue, Span, TraceSummary},
    data::models::HexEncodedId,
};
use zed_extension_api::{
    self as zed, http_client, serde_json, Result, SlashCommand, SlashCommandArgumentCompletion,
    SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

const HTTP_REQUEST_METHOD: &str = "http.request.method";
const FPX_HTTP_REQUEST_PATHNAME: &str = "fpx.http.request.pathname";
const HTTP_RESPONSE_STATUS_CODE: &str = "http.response.status_code";
const FPX_HTTP_REQUEST_ENV: &str = "fpx.http.request.env";
const HTTP_AUTHORIZATION: &str = "http.request.header.authorization";
const HTTP_NEON_CONNECTION_STRING: &str = "http.request.header.neon-connection-string";

// Currently not configurable from the editor
const BASE_URL: &str = "http://localhost:8788";

fn get_traces() -> Result<Vec<TraceSummary>> {
    let url = format!("{}/v1/traces", BASE_URL);

    let request = http_client::HttpRequest {
        method: http_client::HttpMethod::Get,
        url: url.clone(),
        body: None,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        redirect_policy: http_client::RedirectPolicy::NoFollow,
    };

    let response = http_client::fetch(&request).map_err(|e| format!("Failed to fetch: {}", e))?;

    serde_json::from_slice(&response.body).map_err(|e| format!("Failed to parse JSON: {}", e))
}

fn get_spans(trace_id: &str) -> Result<Vec<Span>> {
    let url = format!("{}/v1/traces/{}/spans", BASE_URL, trace_id);

    let request = http_client::HttpRequest {
        method: http_client::HttpMethod::Get,
        url,
        body: None,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        redirect_policy: http_client::RedirectPolicy::NoFollow,
    };

    let response = http_client::fetch(&request).map_err(|e| format!("Failed to fetch: {}", e))?;

    serde_json::from_slice(&response.body).map_err(|e| format!("Failed to parse JSON: {}", e))
}

fn strip_env_variables(mut span: Span) -> Span {
    span.attributes.0.remove(FPX_HTTP_REQUEST_ENV);
    if let Some(auth) = span.attributes.0.get_mut(HTTP_AUTHORIZATION) {
        *auth = Some(AttributeValue::StringValue("*****".to_string()));
    }
    if let Some(neon_conn_string) = span.attributes.0.get_mut(HTTP_NEON_CONNECTION_STRING) {
        *neon_conn_string = Some(AttributeValue::StringValue("*****".to_string()));
    }
    span
}

struct FiberplaneStudioExtension;

impl zed::Extension for FiberplaneStudioExtension {
    fn new() -> Self {
        FiberplaneStudioExtension
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed_extension_api::SlashCommandArgumentCompletion>, String> {
        let traces = get_traces()?;

        match command.name.as_str() {
            "trace" => {
                if traces.len() == 0 {
                    return Ok(vec![SlashCommandArgumentCompletion {
                        label: format!("No traces found"),
                        run_command: false,
                        new_text: format!("No traces found, check if your Fiberplane Studio is running and if there are traces recorded."),
                    }]);
                }

                return Ok(traces
                    .iter()
                    .flat_map(|trace| {
                        trace.spans.iter().filter_map(|span| {
                            let stripped_span = strip_env_variables(span.clone());
                            let name = &stripped_span.name;

                            // Only show top-level requests
                            if name != "request" {
                                return None;
                            }

                            let method = match stripped_span.attributes.0.get(HTTP_REQUEST_METHOD) {
                                Some(Some(AttributeValue::StringValue(s))) => s.to_owned(),
                                _ => "UNKNOWN".to_owned(),
                            };

                            let path =
                                match stripped_span.attributes.0.get(FPX_HTTP_REQUEST_PATHNAME) {
                                    Some(Some(AttributeValue::StringValue(s))) => s,
                                    _ => "/",
                                };
                            let status_code = stripped_span
                                .attributes
                                .0
                                .get(HTTP_RESPONSE_STATUS_CODE)
                                .and_then(|v| v.as_ref())
                                .and_then(|v| match v {
                                    AttributeValue::DoubleValue(n) => n.to_string().into(),
                                    AttributeValue::IntValue(n) => n.to_string().into(),
                                    AttributeValue::StringValue(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .unwrap_or("???".to_string());

                            let label = format!("{} {} {}", status_code, method, path);

                            Some(SlashCommandArgumentCompletion {
                                new_text: trace.trace_id.to_string(),
                                label,
                                run_command: true,
                            })
                        })
                    })
                    .collect::<Vec<_>>());
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "trace" => {
                let trace_id = args.first().ok_or("no trace id provided")?;

                let spans = get_spans(trace_id)?;
                let trace = TraceSummary {
                    trace_id: HexEncodedId::new(trace_id).expect("trace id is invalid"),
                    spans: spans.into_iter().map(strip_env_variables).collect(),
                };
                let formatted_json = serde_json::to_string_pretty(&trace)
                    .map_err(|e| format!("Failed to format JSON: {}", e))?;
                let spans_text = format!("```json\n{}\n```", formatted_json);

                Ok(SlashCommandOutput {
                    sections: vec![SlashCommandOutputSection {
                        range: (0..spans_text.len()).into(),
                        label: format!("Trace: {}", trace_id),
                    }],
                    text: spans_text,
                })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(FiberplaneStudioExtension);
