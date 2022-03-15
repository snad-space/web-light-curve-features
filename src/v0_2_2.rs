use light_curve_feature_0_2_2::*;
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

thread_local! {
    static MAG_FE: FeatureExtractor<f64> = {
        let mut periodogram_feature_evaluator = Periodogram::new(5);
        periodogram_feature_evaluator.set_nyquist(Box::new(AverageNyquistFreq));
        periodogram_feature_evaluator.set_freq_resolution(10.0);
        periodogram_feature_evaluator.set_max_freq_factor(2.0);
        periodogram_feature_evaluator.add_feature(Box::new(Amplitude::default()));
        periodogram_feature_evaluator.add_feature(Box::new(BeyondNStd::new(2.0)));
        periodogram_feature_evaluator.add_feature(Box::new(BeyondNStd::new(3.0)));
        periodogram_feature_evaluator.add_feature(Box::new(StandardDeviation::default()));

        feat_extr!(
            Amplitude::default(),
            AndersonDarlingNormal::default(),
            BeyondNStd::new(1.0), // default
            BeyondNStd::new(2.0),
            Cusum::default(),
            EtaE::default(),
            InterPercentileRange::new(0.02),
            InterPercentileRange::new(0.1),
            InterPercentileRange::new(0.25),
            Kurtosis::default(),
            LinearFit::default(),
            LinearTrend::default(),
            MagnitudePercentageRatio::new(0.4, 0.05), // default
            MagnitudePercentageRatio::new(0.2, 0.05),
            MaximumSlope::default(),
            Mean::default(),
            MedianAbsoluteDeviation::default(),
            MedianBufferRangePercentage::new(0.1),
            MedianBufferRangePercentage::new(0.2),
            PercentAmplitude::default(),
            PercentDifferenceMagnitudePercentile::new(0.05), // default
            PercentDifferenceMagnitudePercentile::new(0.1),
            periodogram_feature_evaluator,
            ReducedChi2::default(),
            Skew::default(),
            StandardDeviation::default(),
            StetsonK::default(),
            WeightedMean::default(),
        )
    };

    static FLUX_FE: FeatureExtractor<f64> =
        feat_extr!(
            AndersonDarlingNormal::default(),
            Cusum::default(),
            EtaE::default(),
            ExcessVariance::default(),
            Kurtosis::default(),
            MeanVariance::default(),
            ReducedChi2::default(),
            Skew::default(),
            StetsonK::default(),
        );

    static FEATURE_NAMES: Vec<String> = {
        let magn_fe_names: Vec<String> = MAG_FE.with(|fe| fe.get_names().iter().map(|s| s.to_string()).collect());
        let flux_fe_names: Vec<String> = FLUX_FE.with(|fe| fe.get_names().iter().map(|s| s.to_string()).collect());
        magn_fe_names
            .iter()
            .map(|name| (name, "magn"))
            .chain(flux_fe_names.iter().map(|name| (name, "flux")))
            .map(|(name, brightness_type)| {
                format!("{}_{}", name, brightness_type)
            })
            .collect()
    };
}

type FeatureValues = HashMap<String, f64>;

#[post("/", format = "json", data = "<data>")]
pub fn index(mut data: Json<Data>) -> Result<Json<FeatureValues>, BadRequest<String>> {
    if data.light_curve.len() < 5 {
        return Err(BadRequest(Some(
            "Bad request: Light curve must have at least five observations".into(),
        )));
    }
    data.light_curve
        .sort_unstable_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

    let (t, mag, err2): (Vec<_>, Vec<_>, Vec<_>) = data
        .light_curve
        .iter()
        .map(|obs| (obs.t, obs.m, obs.err.powi(2)))
        .unzip3();
    let mag_weight: Vec<_> = err2.iter().copied().map(f64::recip).collect();
    let flux: Vec<_> = mag.iter().map(|&m| 10_f64.powf(-0.4 * m)).collect();
    let flux_weight: Vec<_> = flux
        .iter()
        .zip(mag_weight.iter())
        .map(|(f, w_m)| w_m / f64::powi(0.4 * f64::ln(10.0) * f, 2))
        .collect();

    let mut mag_ts = time_series::TimeSeries::new(&t, &mag, Some(&mag_weight));
    let mut flux_ts = time_series::TimeSeries::new(&t, &flux, Some(&flux_weight));

    let values = {
        let mag_values = MAG_FE
            .with(|fe| fe.eval(&mut mag_ts))
            .map_err(|e| BadRequest(Some(format!("Bad request: {:?}", e))))?;
        let flux_values = FLUX_FE
            .with(|fe| fe.eval(&mut flux_ts))
            .map_err(|e| BadRequest(Some(format!("Bad request: {:?}", e))))?;
        [mag_values, flux_values].concat()
    };
    let features: FeatureValues =
        FEATURE_NAMES.with(|names| names.iter().cloned().zip(values.into_iter()).collect());
    Ok(Json(features))
}
