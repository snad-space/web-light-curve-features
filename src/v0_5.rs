use light_curve_feature_0_5::ndarray::{Array1, Zip};
use light_curve_feature_0_5::transformers::{
    arcsinh::ArcsinhTransformer, bazin_fit::BazinFitTransformer, composed::ComposedTransformer,
    identity::IdentityTransformer, lg::LgTransformer, ln1p::Ln1pTransformer,
};
use light_curve_feature_0_5::*;
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize};
use std::collections::HashMap;

pub const MAG_ZP_F64: f64 = 8.9 + 6.0 * 2.5; // μJy

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
        let bins: Feature<f64> = {
            let eta_e: Feature<f64> = {
                let feature: Feature<f64> = EtaE::default().into();
                let transformer: Transformer<f64> = Ln1pTransformer {}.into();
                Transformed::new(feature, transformer).unwrap().into()
            };
            let linear_fit: Feature<f64> = {
                let feature: Feature<f64> = LinearFit::default().into();
                let transformer: Transformer<f64> = ComposedTransformer::new(vec![
                    (ArcsinhTransformer {}.into(), 1), // slope
                    (LgTransformer {}.into(), 1),      // slope sigma
                    (Ln1pTransformer {}.into(), 1),    // reduced chi2
                ])
                .unwrap()
                .into();
                Transformed::new(feature, transformer).unwrap().into()
            };
            let linear_trend: Feature<f64> = {
                let feature: Feature<f64> = LinearTrend::default().into();
                let transformer: Transformer<f64> = ComposedTransformer::new(vec![
                    (ArcsinhTransformer {}.into(), 1), // trend
                    (LgTransformer {}.into(), 1),      // trend sigma
                    (LgTransformer {}.into(), 1),      // noise
                ])
                .unwrap()
                .into();
                Transformed::new(feature, transformer).unwrap().into()
            };
            let maximum_slope: Feature<f64> = {
                let feature: Feature<f64> = MaximumSlope::default().into();
                let transformer: Transformer<f64> = LgTransformer {}.into();
                Transformed::new(feature, transformer).unwrap().into()
            };

            let mut bins = Bins::new(1.0, 0.0);
            bins.add_feature(Cusum::new().into());
            bins.add_feature(eta_e);
            bins.add_feature(linear_fit);
            bins.add_feature(linear_trend);
            bins.add_feature(maximum_slope);

            bins.into()
        };

        let inter_percentile_range_02: Feature<f64> = {
            let feature = InterPercentileRange::new(0.02).into();
            let transformer: Transformer<f64> = LgTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };
        let inter_percentile_range_10: Feature<f64> = {
            let feature = InterPercentileRange::new(0.1).into();
            let transformer: Transformer<f64> = LgTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };
        let inter_percentile_range_25: Feature<f64> = {
            let feature = InterPercentileRange::new(0.25).into();
            let transformer: Transformer<f64> = LgTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };

        let periodogram: Feature<f64> = {
            let mut periodogram = Periodogram::new(5);
            periodogram.set_nyquist(NyquistFreq::fixed(24.0));
            periodogram.set_freq_resolution(10.0);
            periodogram.set_max_freq_factor(2.0);
            periodogram.into()
        };

        let otsu_split: Feature<f64> = {
            let feature = OtsuSplit::new().into();
            let transformer: Transformer<f64> = ComposedTransformer::new(vec![
                (LgTransformer {}.into(), 1),       // mean diff
                (IdentityTransformer {}.into(), 1), // std lower
                (IdentityTransformer {}.into(), 1), // std upper
                (IdentityTransformer {}.into(), 1), // lower to all ratio
            ])
            .unwrap()
            .into();
            Transformed::new(feature, transformer).unwrap().into()
        };

        let reduced_chi2: Feature<f64> = {
            let feature = ReducedChi2::new().into();
            let transformer: Transformer<f64> = Ln1pTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };

        let skew: Feature<f64> = {
            let feature = Skew::new().into();
            let transformer: Transformer<f64> = ArcsinhTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };

        FeatureExtractor::from_features(vec![
            BeyondNStd::new(1.0).into(), // default
            BeyondNStd::new(2.0).into(),
            bins,
            inter_percentile_range_02,
            inter_percentile_range_25,
            inter_percentile_range_10,
            Kurtosis::new().into(),
            otsu_split,
            periodogram.into(),
            reduced_chi2,
            skew,
            StetsonK::new().into(),
            WeightedMean::new().into(),
        ])
        .into()
    };

    static FLUX_FE: FeatureExtractor<f64, Feature<f64>> =
        {        let anderson_darling_normal: Feature<f64> = {
            let feature = AndersonDarlingNormal::default().into();
            let transformer: Transformer<f64> = Ln1pTransformer {}.into();
            Transformed::new(feature, transformer).unwrap().into()
        };

        let bazin_fit: Feature<f64> = {
            let inits_bounds = BazinInitsBounds::option_arrays(
                [None; 5],
                [
                    Some(f64::powf(10.0, -0.4 * (30.0 - MAG_ZP_F64))), // amplitude
                    None,                                              // baseline
                    None,                                              // t0
                    Some(1e-4),                                        // rise time
                    Some(1e-4),                                        // fall time
                ],
                [
                    Some(f64::powf(10.0, -0.4 * (0.0 - MAG_ZP_F64))), // amplitude
                    None,                                             // baseline
                    None,                                             // t0
                    Some(3e4),                                        // rise time
                    Some(3e4),                                        // fall time
                ],
            );

            let fit = BazinFit::new(
                CeresCurveFit::new(20, None).into(),
                LnPrior::none(),
                inits_bounds,
            );
            let feature: Feature<f64> = fit.into();
            let transformer = Transformer::BazinFit(BazinFitTransformer::new(MAG_ZP_F64));
            let transformed = Transformed::new(feature, transformer).unwrap();
            transformed.into()
        };

        FeatureExtractor::from_features(vec![
            anderson_darling_normal,
            bazin_fit,
            ExcessVariance::new().into(),
        ])
        .into()};

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

fn data_to_time_series(mut data: Data) -> Result<TimeSeries<'static, f64>, BadRequest<String>> {
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

    Ok(TimeSeries::new(time, mag, mag_weight))
}

fn flux_ts_from_mag_ts<'a>(mag_ts: &'a TimeSeries<'_, f64>, zp: f64) -> TimeSeries<'a, f64> {
    let flux = mag_ts.m.sample.mapv(|m| 10_f64.powf(-0.4 * (m - zp)));
    let flux_weight = {
        let mut flux_weight = Array1::zeros(mag_ts.lenu());
        Zip::from(&mut flux_weight)
            .and(&flux)
            .and(&mag_ts.w.sample)
            .for_each(|f_w, &f, &w_m| *f_w = w_m / f64::powi(0.4 * f64::ln(10.0) * f, 2));
        flux_weight
    };

    TimeSeries::new(mag_ts.t.sample.view(), flux, flux_weight)
}

#[post("/", format = "json", data = "<data>")]
pub fn index(data: Json<Data>) -> Result<Json<FeatureValues>, BadRequest<String>> {
    let mut mag_ts = data_to_time_series(data.0)?;
    let mag_values = MAG_FE
        .with(|fe| fe.eval(&mut mag_ts))
        .map_err(|e| BadRequest(Some(format!("Bad request: {:?}", e))))?;

    let mut flux_ts = flux_ts_from_mag_ts(&mag_ts, MAG_ZP_F64);
    let flux_values = FLUX_FE
        .with(|fe| fe.eval(&mut flux_ts))
        .map_err(|e| BadRequest(Some(format!("Bad request: {:?}", e))))?;

    let values = [mag_values, flux_values].concat();

    let features: FeatureValues =
        FEATURE_NAMES.with(|names| names.iter().cloned().zip(values.into_iter()).collect());
    Ok(Json(features))
}
