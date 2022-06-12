use light_curve_feature_0_4::*;
use ndarray::{Array1, Zip};
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize};
use std::collections::HashMap;

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
    static MAG_FE: FeatureExtractor<f64, Feature<f64>> = {
        let mut periodogram_feature_evaluator = Periodogram::new(5);
        periodogram_feature_evaluator.set_nyquist(FixedNyquistFreq::from_dt(1.0 / 24.0).into());
        periodogram_feature_evaluator.set_freq_resolution(10.0);
        periodogram_feature_evaluator.set_max_freq_factor(1.0);
        periodogram_feature_evaluator.add_feature(Amplitude::default().into());
        periodogram_feature_evaluator.add_feature(BeyondNStd::new(2.0).into());
        periodogram_feature_evaluator.add_feature(BeyondNStd::new(3.0).into());
        periodogram_feature_evaluator.add_feature(StandardDeviation::default().into());

        FeatureExtractor::from_features(vec![
            Amplitude::default().into(),
            AndersonDarlingNormal::default().into(),
            BeyondNStd::new(1.0).into(), // default
            BeyondNStd::new(2.0).into(),
            Cusum::default().into(),
            InterPercentileRange::new(0.02).into(),
            InterPercentileRange::new(0.1).into(),
            InterPercentileRange::new(0.25).into(),
            Kurtosis::default().into(),
            LinearFit::default().into(),
            LinearTrend::default().into(),
            MagnitudePercentageRatio::new(0.4, 0.05).into(), // default
            MagnitudePercentageRatio::new(0.2, 0.05).into(),
            Mean::default().into(),
            MedianAbsoluteDeviation::default().into(),
            MedianBufferRangePercentage::new(0.1).into(),
            MedianBufferRangePercentage::new(0.2).into(),
            PercentAmplitude::default().into(),
            PercentDifferenceMagnitudePercentile::new(0.05).into(), // default
            PercentDifferenceMagnitudePercentile::new(0.1).into(),
            periodogram_feature_evaluator.into(),
            ReducedChi2::default().into(),
            Skew::default().into(),
            StandardDeviation::default().into(),
            StetsonK::default().into(),
            WeightedMean::default().into(),
        ])
    };

    static FLUX_FE: FeatureExtractor<f64, Feature<f64>> =
        FeatureExtractor::from_features(vec![
            AndersonDarlingNormal::default().into(),
            Cusum::default().into(),
            ExcessVariance::default().into(),
            Kurtosis::default().into(),
            MeanVariance::default().into(),
            ReducedChi2::default().into(),
            Skew::default().into(),
            StetsonK::default().into(),
        ]);

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
    let n_obs = data.light_curve.len();

    if n_obs < 5 {
        return Err(BadRequest(Some(
            "Bad request: Light curve must have at least five observations".into(),
        )));
    }

    data.light_curve
        .sort_unstable_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

    let (time, mag, mag_weight) = {
        let mut t = Array1::zeros(n_obs);
        let mut mag = Array1::zeros(n_obs);
        let mut mag_weight = Array1::zeros(n_obs);
        Zip::from(&data.light_curve)
            .and(&mut t)
            .and(&mut mag)
            .and(&mut mag_weight)
            .for_each(|obs, t, m, m_w| {
                *t = obs.t;
                *m = obs.m;
                *m_w = obs.err.powi(-2);
            });
        (t, mag, mag_weight)
    };
    let flux = mag.mapv(|m| 10_f64.powf(-0.4 * m));
    let flux_weight = {
        let mut flux_weight = Array1::zeros(n_obs);
        Zip::from(&mut flux_weight)
            .and(&flux)
            .and(&mag_weight)
            .for_each(|f_w, &f, &w_m| *f_w = w_m / f64::powi(0.4 * f64::ln(10.0) * f, 2));
        flux_weight
    };

    let mut mag_ts = TimeSeries::new(time.view(), mag, mag_weight);
    let mut flux_ts = TimeSeries::new(time.view(), flux, flux_weight);

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
