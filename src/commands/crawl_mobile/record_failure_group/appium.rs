use super::*;

impl Appium {
    pub(crate) fn new(base: &str) -> Result<Self> {
        Ok(Self {
            base: canonical_driver_url(base)?,
            // Zero redirects: an Appium reply may not send this worker, its
            // session id or its screenshots to another origin (finding 12b).
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(45))
                .redirects(0)
                .build(),
        })
    }

    pub(crate) fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let request = match method {
            "GET" => self.agent.get(&url),
            "POST" => self.agent.post(&url),
            "DELETE" => self.agent.delete(&url),
            _ => bail!("unsupported Appium method {method}"),
        };
        let response = match if let Some(body) = body {
            request.send_json(body.clone())
        } else {
            request.call()
        } {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let diagnostic = response
                    .into_string()
                    .unwrap_or_else(|error| format!("<unreadable response body: {error}>"));
                bail!("Appium {method} {path} returned HTTP {status}: {diagnostic}");
            }
            Err(ureq::Error::Transport(error)) => {
                bail!("Appium {method} {path} transport failed: {error}");
            }
        };
        if (300..400).contains(&response.status()) {
            bail!(
                "Appium {method} {path} returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        // `into_json` has no length bound, so a screenshot or screen-recording
        // body would be materialized whole inside a Value before any decode
        // could reject it (finding 12).
        let limit = response_limit(path);
        serde_json::from_reader(response.into_reader().take(limit)).map_err(|error| {
            anyhow!(
                "Appium {method} {path} returned invalid JSON within its {limit}-byte response bound: {error}"
            )
        })
    }

    pub(crate) fn create_session(
        &self,
        platform: Platform,
        app_id: &str,
        execution: &super::crawl::RuntimeExecutionIdentity,
    ) -> Result<(String, Value)> {
        let mut always = serde_json::Map::new();
        always.insert("platformName".into(), json!(platform.appium_name()));
        always.insert("appium:automationName".into(), json!(platform.automation()));
        always.insert(platform.app_key().into(), json!(app_id));
        always.insert("appium:noReset".into(), json!(true));
        always.insert("appium:autoGrantPermissions".into(), json!(false));
        always.insert("appium:autoAcceptAlerts".into(), json!(false));
        always.insert("appium:autoDismissAlerts".into(), json!(false));
        always.insert("appium:newCommandTimeout".into(), json!(180));
        let device_id = execution
            .device_id
            .as_deref()
            .context("mobile execution identity has no exact UDID")?;
        always.insert("appium:udid".into(), json!(device_id));
        let payload = json!({ "capabilities": { "alwaysMatch": always } });
        let response = self.call("POST", "/session", Some(&payload))?;
        let session = response
            .pointer("/value/sessionId")
            .or_else(|| response.get("sessionId"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium did not return a session id: {response}"))?;
        let capabilities = response
            .pointer("/value/capabilities")
            .or_else(|| response.get("capabilities"))
            .cloned()
            .ok_or_else(|| anyhow!("Appium did not return session capabilities: {response}"))?;
        Ok((session, capabilities))
    }

    pub(crate) fn source(&self, session: &str) -> Result<String> {
        self.call("GET", &format!("/session/{session}/source"), None)?
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium returned no accessibility source"))
    }

    pub(crate) fn active_app_identity(&self, session: &str) -> Result<String> {
        let response = self.call(
            "POST",
            &format!("/session/{session}/execute/sync"),
            Some(&json!({"script": "mobile: activeAppInfo", "args": []})),
        )?;
        response
            .pointer("/value/bundleId")
            .or_else(|| response.pointer("/value/package"))
            .or_else(|| response.pointer("/value/appPackage"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium activeAppInfo returned no bundle/package identity"))
    }

    pub(crate) fn screenshot(&self, session: &str) -> Result<Vec<u8>> {
        let value = self
            .call("GET", &format!("/session/{session}/screenshot"), None)?
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Appium returned no screenshot"))?
            .to_string();
        STANDARD
            .decode(value)
            .context("Appium screenshot was not base64")
    }
    pub(crate) fn alert_text(&self, session: &str) -> Result<Option<String>> {
        let path = format!("/session/{session}/alert/text");
        let response = match self.agent.get(&format!("{}{}", self.base, path)).call() {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let body = response
                    .into_string()
                    .unwrap_or_else(|error| format!("<unreadable response body: {error}>"));
                if serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|value| {
                        value
                            .pointer("/value/error")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .as_deref()
                    == Some("no such alert")
                {
                    return Ok(None);
                }
                bail!("Appium GET {path} returned HTTP {status}: {body}");
            }
            Err(ureq::Error::Transport(error)) => {
                bail!("Appium GET {path} transport failed: {error}");
            }
        };
        if (300..400).contains(&response.status()) {
            bail!(
                "Appium GET {path} returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        let limit = response_limit(&path);
        let value: Value = serde_json::from_reader(response.into_reader().take(limit))
            .with_context(|| {
                format!("Appium GET {path} alert text exceeded its {limit}-byte bound or was invalid JSON")
            })?;
        Ok(value
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string))
    }

    pub(crate) fn start_recording(&self, session: &str) -> Result<()> {
        self.call(
            "POST",
            &format!("/session/{session}/appium/start_recording_screen"),
            Some(&json!({"options": {"timeLimit": 180}})),
        )?;
        Ok(())
    }

    pub(crate) fn stop_recording(&self, session: &str) -> Result<Option<Vec<u8>>> {
        let response = self.call(
            "POST",
            &format!("/session/{session}/appium/stop_recording_screen"),
            Some(&json!({})),
        )?;
        let Some(encoded) = response.get("value").and_then(Value::as_str) else {
            return Ok(None);
        };
        Ok(Some(
            STANDARD
                .decode(encoded)
                .context("Appium screen recording was not base64")?,
        ))
    }

    pub(crate) fn element(&self, session: &str, selector: &str) -> Result<String> {
        let found = self.call(
            "POST",
            &format!("/session/{session}/element"),
            Some(&json!({"using": "xpath", "value": selector})),
        )?;
        let value = found
            .get("value")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("Appium found no element for {selector}"))?;
        value
            .get("element-6066-11e4-a52e-4f735466cecf")
            .or_else(|| value.get("ELEMENT"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium element response had no id"))
    }

    /// Poll the accessibility source until two consecutive reads agree. A fixed
    /// 700ms sleep was the only settling wait before a screenshot, so a slower
    /// transition was captured mid-animation; 700ms is now the floor of a
    /// bounded wait, not the whole wait (finding 19).
    pub(crate) fn settle(&self, session: &str) -> Result<String> {
        let started = Instant::now();
        let mut previous = hash_text(&self.source(session)?);
        loop {
            std::thread::sleep(Duration::from_millis(200));
            let current = self.source(session)?;
            let digest = hash_text(&current);
            if digest == previous && started.elapsed() >= Duration::from_millis(700) {
                return Ok(current);
            }
            if started.elapsed() >= Duration::from_secs(20) {
                return Err(anyhow::Error::new(RecordFailure {
                    code: "mobile_surface_never_settled",
                    message:
                        "the mobile surface produced no two consecutive identical accessibility sources within 20s"
                            .to_string(),
                }));
            }
            previous = digest;
        }
    }

    pub(crate) fn click(&self, session: &str, selector: &str) -> Result<()> {
        let element = self.element(session, selector)?;
        self.call(
            "POST",
            &format!("/session/{session}/element/{element}/click"),
            Some(&json!({})),
        )?;
        self.settle(session)?;
        Ok(())
    }

    pub(crate) fn delete_session(&self, session: &str) {
        let _ = self.call("DELETE", &format!("/session/{session}"), None);
    }
}
