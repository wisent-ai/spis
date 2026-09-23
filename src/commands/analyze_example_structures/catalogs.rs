use super::*;

pub(crate) const CATALOGS: &[&str] = &[
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "landing-page-examples",
    "wisent-product-examples",
];

/// Python's round(): halves go to the nearest even value.
pub(crate) fn py_round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    let scaled = value * factor;
    let floor = scaled.floor();
    let fract = scaled - floor;
    let rounded = if (fract - 0.5).abs() < 1e-9 {
        if floor % 2.0 == 0.0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        scaled.round()
    };
    rounded / factor
}

pub(crate) struct Gray {
    pub(crate) data: Vec<f32>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Gray {
    pub(crate) fn at(&self, y: usize, x: usize) -> f32 {
        self.data[y * self.width + x]
    }
}

/// np.convolve(values, kernel, mode="same") with a uniform kernel: zero-padded
/// at both ends, each output divided by the full kernel width.
pub(crate) fn moving_average(values: &[f32], radius: usize) -> Vec<f32> {
    let width = radius * 2 + 1;
    if values.len() < width {
        return values.to_vec();
    }
    values
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mut sum = 0.0f32;
            for k in 0..width {
                let j = i + k;
                if j >= radius && j - radius < values.len() {
                    sum += values[j - radius];
                }
            }
            sum / width as f32
        })
        .collect()
}

/// numpy.percentile(values, q) with linear interpolation.
pub(crate) fn percentile_linear(mut values: Vec<f32>, q: f64) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let position = (n - 1) as f64 * q;
    let lower = position.floor() as usize;
    let upper = std::cmp::min(lower + 1, n - 1);
    values[lower] as f64 + (values[upper] - values[lower]) as f64 * (position - lower as f64)
}

pub(crate) fn mean(values: &[f32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() as f64 / values.len() as f64
}

/// Population standard deviation (numpy default ddof=0).
pub(crate) fn stddev(values: &[f32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mu = mean(values);
    let var = values.iter().map(|v| (*v as f64 - mu).powi(2)).sum::<f64>() / values.len() as f64;
    var.sqrt()
}

pub(crate) fn separator_positions(gray: &Gray, vertical: bool) -> Vec<Value> {
    // Per-coordinate mean absolute neighbor difference along the scan axis.
    let differences: Vec<f32> = if vertical {
        let mut diffs = Vec::with_capacity((gray.width - 1).max(0));
        for x in 0..gray.width.saturating_sub(1) {
            let s: f32 = (0..gray.height)
                .map(|y| (gray.at(y, x + 1) - gray.at(y, x)).abs())
                .sum();
            diffs.push(s / gray.height as f32);
        }
        diffs
    } else {
        let mut diffs = Vec::with_capacity((gray.height - 1).max(0));
        for y in 0..gray.height.saturating_sub(1) {
            let s: f32 = (0..gray.width)
                .map(|x| (gray.at(y + 1, x) - gray.at(y, x)).abs())
                .sum();
            diffs.push(s / gray.width as f32);
        }
        diffs
    };
    let extent = if vertical { gray.width } else { gray.height };

    let smoothed = moving_average(&differences, std::cmp::max(1, extent / 350));
    let low = ((extent as f64) * 0.06) as usize;
    let high = ((extent as f64) * 0.94) as usize;
    if low >= high || high > smoothed.len() {
        return Vec::new();
    }
    let segment = &smoothed[low..high];
    if segment.is_empty() {
        return Vec::new();
    }
    let threshold =
        percentile_linear(segment.to_vec(), 0.91).max(mean(segment) + stddev(segment) * 1.15);

    let group_gap = std::cmp::max(2, extent / 180);
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for index in low..high {
        if smoothed[index] < threshold as f32 {
            continue;
        }
        match groups.last_mut() {
            Some(group) if index - group[group.len() - 1] <= group_gap => group.push(index),
            _ => groups.push(vec![index]),
        }
    }

    // Strongest index per group (first maximal one on ties), then take up to 4
    // peaks by descending score, keeping peaks at least 7.5% of the extent apart.
    let mut peaks: Vec<(usize, f64)> = groups
        .iter()
        .map(|group| {
            let best = *group
                .iter()
                .max_by(|a, b| smoothed[**a].partial_cmp(&smoothed[**b]).unwrap())
                .unwrap();
            (best, smoothed[best] as f64)
        })
        .collect();
    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let min_distance = (extent as f64) * 0.075;
    let mut selected: Vec<(usize, f64)> = Vec::new();
    for (index, score) in peaks {
        if selected
            .iter()
            .all(|(chosen, _)| (index as f64 - *chosen as f64).abs() >= min_distance)
        {
            selected.push((index, score));
        }
        if selected.len() == 4 {
            break;
        }
    }

    let maximum = selected
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);
    if !(maximum > 0.0) {
        return Vec::new();
    }
    selected.sort_by_key(|(index, _)| *index);
    selected
        .into_iter()
        .map(|(index, score)| {
            json!({
                "position": py_round(index as f64 / extent as f64, 3),
                "strength": py_round(score / maximum, 3),
            })
        })
        .collect()
}

pub(crate) fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

pub(crate) struct Hints {
    pub(crate) leading: bool,
    pub(crate) trailing: bool,
    pub(crate) table: bool,
    pub(crate) canvas: bool,
    pub(crate) command: bool,
    pub(crate) mobile: bool,
    pub(crate) request_response: bool,
}
