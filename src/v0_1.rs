use lazy_static::lazy_static;
use light_curve_feature_0_1::*;
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize};
use std::collections::HashMap;
use unzip3::Unzip3;

#[derive(Debug, Deserialize)]
pub struct Data {
    light_curve: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    t: f64,
    m: f64,
    err: f64,
}

struct TruncMedianNyquistFreq {
    m: MedianNyquistFreq,
    max_freq: f64,
}

impl TruncMedianNyquistFreq {
    fn new(min_dt: f64) -> Self {
        Self {
            m: MedianNyquistFreq,
            max_freq: std::f64::consts::PI / min_dt,
        }
    }
}

impl NyquistFreq<f64> for TruncMedianNyquistFreq {
    fn nyquist_freq(&self, t: &[f64]) -> f64 {
        f64::min(self.m.nyquist_freq(t), self.max_freq)
    }
}

lazy_static! {
    static ref FE: FeatureExtractor<f64> = {
        let mut periodogram_feature_evaluator = Periodogram::new(3);
        periodogram_feature_evaluator.set_nyquist(Box::new(TruncMedianNyquistFreq::new(
            300.0 / 86400.0,
        )));
        periodogram_feature_evaluator.set_max_freq_factor(2.0);
        periodogram_feature_evaluator.add_features(vec![
            Box::new(Amplitude::default()),
            Box::new(BeyondNStd::default()),
            Box::new(BeyondNStd::new(2.0)),
            Box::new(Cusum::default()),
            Box::new(Eta::default()),
            Box::new(InterPercentileRange::default()),
            Box::new(StandardDeviation::default()),
            Box::new(PercentAmplitude::default()),
        ]);
        feat_extr!(
            Amplitude::default(),
            BeyondNStd::default(),
            BeyondNStd::new(2.0),
            Cusum::default(),
            Eta::default(),
            EtaE::default(),
            InterPercentileRange::default(),
            InterPercentileRange::new(0.1),
            Kurtosis::default(),
            LinearFit::default(),
            LinearTrend::default(),
            MagnitudePercentageRatio::new(0.4, 0.05), // default
            MagnitudePercentageRatio::new(0.2, 0.1),
            MaximumSlope::default(),
            Mean::default(),
            MedianAbsoluteDeviation::default(),
            MedianBufferRangePercentage::new(0.05), // not default
            PercentAmplitude::default(),
            PercentDifferenceMagnitudePercentile::new(0.05), // default
            PercentDifferenceMagnitudePercentile::new(0.2),
            periodogram_feature_evaluator,
            ReducedChi2::default(),
            Skew::default(),
            StandardDeviation::default(),
            StetsonK::default(),
            WeightedMean::default(),
        )
    };
}

type FeatureValues = HashMap<String, f64>;

#[post("/", format = "json", data = "<data>")]
pub fn index(mut data: Json<Data>) -> Result<Json<FeatureValues>, BadRequest<&'static str>> {
    if data.light_curve.len() < 5 {
        return Err(BadRequest(Some(
            "Bad request: Light curve must have at least five observations",
        )));
    }
    data.light_curve
        .sort_unstable_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    let (t, m, err2): (Vec<_>, Vec<_>, Vec<_>) = data
        .light_curve
        .iter()
        .map(|obs| (obs.t, obs.m, obs.err.powi(2)))
        .unzip3();
    let ts = time_series::TimeSeries::new(&t[..], &m[..], Some(&err2[..]));
    let names = FE.get_names();
    let values = FE.eval(ts);
    let features: FeatureValues = names
        .into_iter()
        .map(String::from)
        .zip(values.into_iter())
        .collect();
    Ok(Json(features))
}
