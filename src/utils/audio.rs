// yeah ignore this

pub(crate) struct WavAudio {
    samples: Vec<i32>,
    bitrate: u32,
}

fn i32_to_usize_re_lu(v:i32)->usize{
    if v < 0 {
        return 0;
    } else {
        return v as usize;
    }
}

impl WavAudio {
    pub(crate) fn new(filename: &str) -> Result<WavAudio, Box<dyn std::error::Error>>{
        let mut reader = hound::WavReader::open(filename).unwrap();
        let spec = reader.spec();
        let combined_bitrate = spec.sample_rate as u32;
        let mut combined_samples: Vec<i32> = Vec::new();
        // if the polarity is reversed shame on you shame on you
        let num_channels = spec.channels as usize;
    
        for sample_chunk in reader.samples::<i32>().collect::<Result<Vec<_>, _>>()?.chunks(num_channels) {
            let combined_sample = sample_chunk.iter().sum::<i32>() / num_channels as i32;
            combined_samples.push(combined_sample);
        }
        println!("{}, {}, {}, {}", combined_samples.len(), combined_bitrate, spec.sample_rate, spec.bits_per_sample);
    
        return Ok(WavAudio {
            samples:combined_samples,
            bitrate:combined_bitrate
        })
    }

    pub(crate) fn get_index_from_secs(&self, secs: f64) -> i32 {
        return (secs * (self.bitrate as f64)).floor() as i32;
    }
    // note: if OOB, to prevent stochastic behavior in the FFT function we are supposed to pad it with zeros
    fn get_slice(&self, b: i32, e: i32) -> &[i32] {
        let b2 = i32_to_usize_re_lu(b);
        let e2 = i32_to_usize_re_lu(e);
        let max_size = self.samples.len();
        &self.samples[max_size.min(b2)..max_size.min(e2)]
    }

    fn get_slice_back(&self, point: i32, look_back: i32) -> &[i32] {
        return self.get_slice(point - look_back, point);
    }

    pub(crate) fn get_slice_back_seconds(&self, point: f64, look_back: f64) -> &[i32] {
        return self.get_slice_back(self.get_index_from_secs(point), self.get_index_from_secs(look_back));
    }

}
