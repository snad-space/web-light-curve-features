#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use light_curve_feature::*;
use rocket::response::status;
use rocket_contrib::json::Json;
use serde::Deserialize;
use std::collections::HashMap;
use unzip3::Unzip3;

#[derive(Debug, Deserialize)]
struct Data {
    light_curve: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    t: f64,
    m: f64,
    err: f64,
}

fn get_fe() -> FeatureExtractor<f64> {
    let mut periodogram_feature_evaluator = Periodogram::new(3);
    periodogram_feature_evaluator.set_nyquist(Box::new(QuantileNyquistFreq { quantile: 0.1 }));
    periodogram_feature_evaluator.add_features(vec![
        Box::new(Amplitude::default()),
        Box::new(BeyondNStd::default()),
        Box::new(BeyondNStd::new(2.0)),
        Box::new(Cusum::default()),
        Box::new(Eta::default()),
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
        Kurtosis::default(),
        LinearFit::default(),
        LinearTrend::default(),
        MagnitudePercentageRatio::new(0.4, 0.05), // default
        MagnitudePercentageRatio::new(0.2, 0.1),
        MaximumSlope::default(),
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
    )
}

type FeatureValues = HashMap<String, f64>;

#[post("/", format = "json", data = "<data>")]
fn index(mut data: Json<Data>) -> Result<Json<FeatureValues>, status::BadRequest<&'static str>> {
    if data.light_curve.len() < 5 {
        return Err(status::BadRequest(Some(
            "Bad request: Light curve must have at least five observations",
        )));
    }
    data.light_curve
        .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    let (t, m, err2): (Vec<_>, Vec<_>, Vec<_>) = data
        .light_curve
        .iter()
        .map(|obs| (obs.t, obs.m, obs.err.powi(2)))
        .unzip3();
    let ts = time_series::TimeSeries::new(&t[..], &m[..], Some(&err2[..]));
    let fe = get_fe();
    let names = fe.get_names();
    let values = fe.eval(ts);
    let features: FeatureValues = names
        .into_iter()
        .map(|s| String::from(s))
        .zip(values.into_iter())
        .collect();
    Ok(Json(features))
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let path = format!("/api/v{}/", version);
    rocket::ignite()
        .mount("/", routes![index])
        .mount(&path, routes![index])
        .launch();
}
