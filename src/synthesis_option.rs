use jbonsai::{engine::Condition, model::interporation_weight::WeightError};

/// Voice synthesis option.
#[napi(object)]
pub struct SynthesisOption {
  /// Frequency warping parameter alpha
  /// 0.0<=all_pass_constant<=1.0.
  pub all_pass_constant: Option<f64>,

  /// Postfiltering coefficient parameter beta
  /// Default is 0.0. 0.0<=postfiltering_coefficient<=1.0.
  pub postfiltering_coefficient: Option<f64>,

  /// Speech speed
  /// Default is 1.0. 0<=speech_speed_rate. Warning: Do not set a very small value as it consumes CPU time.
  pub speech_speed_rate: Option<f64>,

  /// Additional half tone
  /// Default is 0.0.
  pub additional_half_tone: Option<f64>,

  /// MSD threshold for Stream #1
  /// Default is 0.5. 0.0<=voiced_unvoiced_threshold<=1.0.
  pub voiced_unvoiced_threshold: Option<f64>,

  /// GV weight for Stream #0
  /// Default is 1.0. 0.0<=weight_of_gv_for_spectrum.
  pub weight_of_gv_for_spectrum: Option<f64>,

  /// GV weight for Stream #1
  /// Default is 1.0. 0.0<=weight_of_gv_for_log_f0.
  pub weight_of_gv_for_log_f0: Option<f64>,

  /// Volume in dB
  /// Default is 0.0.
  pub volume_in_db: Option<f64>,

  /// Interporation weights
  pub interporation_weight: Option<InterporationWeight>,
}

/// How loaded models are mixed.
///
/// All weight array must:
/// - be same length as loadad models.
/// - have values between 0.0 and 1.0.
/// - sum up to 1.0.
#[napi(object)]
pub struct InterporationWeight {
  /// Duration
  pub duration: Option<Vec<f64>>,
  /// Stream #0
  pub spectrum: Option<Vec<f64>>,
  /// Stream #1
  pub log_f0: Option<Vec<f64>>,
  /// Stream #2
  pub lpf: Option<Vec<f64>>,
}

impl SynthesisOption {
  pub fn apply_to_engine(&self, condition: &mut Condition) -> Result<(), WeightError> {
    if let Some(alpha) = self.all_pass_constant {
      condition.set_alpha(alpha);
    }
    if let Some(beta) = self.postfiltering_coefficient {
      condition.set_beta(beta);
    }
    if let Some(speech_speed_rate) = self.speech_speed_rate {
      condition.set_speed(speech_speed_rate);
    }
    if let Some(additional_half_tone) = self.additional_half_tone {
      condition.set_additional_half_tone(additional_half_tone);
    }
    if let Some(voiced_unvoiced_threshold) = self.voiced_unvoiced_threshold {
      condition.set_msd_threshold(1, voiced_unvoiced_threshold);
    }
    if let Some(weight_of_gv_for_spectrum) = self.weight_of_gv_for_spectrum {
      condition.set_gv_weight(0, weight_of_gv_for_spectrum);
    }
    if let Some(weight_of_gv_for_log_f0) = self.weight_of_gv_for_log_f0 {
      condition.set_gv_weight(1, weight_of_gv_for_log_f0);
    }
    if let Some(volume_in_db) = self.volume_in_db {
      condition.set_volume(volume_in_db);
    }

    if let Some(ref weights) = self.interporation_weight {
      let iw = condition.get_interporation_weight_mut();
      if let Some(ref duration) = weights.duration {
        iw.set_duration(duration)?;
      }
      if let Some(ref spectrum) = weights.spectrum {
        iw.set_parameter(0, spectrum)?;
        iw.set_gv(0, spectrum)?;
      }
      if let Some(ref log_f0) = weights.log_f0 {
        iw.set_parameter(0, log_f0)?;
        iw.set_gv(0, log_f0)?;
      }
      if let Some(ref lpf) = weights.lpf {
        iw.set_parameter(0, lpf)?;
        iw.set_gv(0, lpf)?;
      }
    }

    Ok(())
  }
}
