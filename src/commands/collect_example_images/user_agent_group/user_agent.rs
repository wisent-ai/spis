use super::*;

pub(crate) const USER_AGENT: &str = crate::USER_AGENT;

#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    pub(crate) url: String,
    pub(crate) hint: String,
    pub(crate) order: usize,
    pub(crate) origin: &'static str,
}

pub(crate) struct Probe {
    pub(crate) format: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) payload: Vec<u8>,
    pub(crate) final_url: String,
}

// ---------------------------------------------------------------------------
// Fetch helpers

pub(crate) struct Fetched {
    pub(crate) data: Vec<u8>,
    /// The media type the server declared, without its parameters; `None`
    /// when the response carried no Content-Type.
    pub(crate) content_type: Option<String>,
    pub(crate) final_url: String,
}

/// GET `url` and read the whole body; no response size is chosen here. The
/// request and the body read each say on stderr what they wait for.
pub(crate) fn fetch(url: &str, accept: &str) -> Result<Fetched> {
    let request = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", accept);
    let resp = stado_wait::blocking(
        stado_wait::Kind::Network,
        "the answer to GET",
        url,
        move || request.call(),
    )
    .map_err(|e| anyhow!("GET {url}: {e}"))?;
    let content_type = resp.header("Content-Type").map(|declared| {
        match declared.split_once(';') {
            Some((media, _parameters)) => media,
            None => declared,
        }
        .trim()
        .to_string()
    });
    let final_url = resp.get_url().to_string();
    let mut data = Vec::new();
    let mut reader = resp.into_reader();
    stado_wait::blocking(stado_wait::Kind::Network, "the response body", url, || {
        reader.read_to_end(&mut data)
    })
    .with_context(|| format!("read body of {url}"))?;
    Ok(Fetched {
        data,
        content_type,
        final_url,
    })
}

// ---------------------------------------------------------------------------
// Image header

/// The format and dimensions of `data` as the `image` crate reads them from
/// its header, without decoding pixels. The format is named by its first file
/// extension, as `reference-record add` names it. Bytes that are no image
/// format the crate reads, or whose header it cannot read, are refused with
/// the crate's cause.
pub(crate) fn image_header(data: &[u8]) -> Result<(&'static str, u32, u32)> {
    let format = image::guess_format(data).context("the bytes are no image format Spis reads")?;
    let name = format
        .extensions_str()
        .first()
        .copied()
        .with_context(|| format!("the image format {format:?} names no file extension"))?;
    let (width, height) = image::ImageReader::with_format(std::io::Cursor::new(data), format)
        .into_dimensions()
        .with_context(|| format!("read the {name} header"))?;
    Ok((name, width, height))
}
