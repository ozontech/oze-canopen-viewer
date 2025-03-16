use oze_canopen::interface::CanOpenInfo;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::sleep};

pub const RATES_LEN: usize = 1024;
pub const RATE_PERIOD: usize = 4;

pub type RatesData = Vec<[f64; 2]>;

#[derive(Clone, Debug)]
pub struct Bitrate {
    rates: Arc<Mutex<RatesData>>,
    data: VecDeque<(f64, usize)>,
    canopen_info: Arc<Mutex<CanOpenInfo>>,
}

impl Bitrate {
    pub fn new(canopen_info: Arc<Mutex<CanOpenInfo>>, output: Arc<Mutex<RatesData>>) -> Self {
        Self {
            data: VecDeque::new(),
            canopen_info,
            rates: output,
        }
    }

    async fn calculate_rate(&self) {
        let mut rates = self.rates.lock().await;
        rates.clear();
        for (i, &(current_time, current_bits)) in self.data.iter().enumerate() {
            if i < RATE_PERIOD {
                continue;
            }

            let (prev_time, prev_bits) = self.data[i - RATE_PERIOD];
            let Ok(bits_diff) = i32::try_from(current_bits - prev_bits) else {
                continue;
            };
            let bits_diff = f64::from(bits_diff);
            let duration_secs = current_time - prev_time;
            if duration_secs > 0.0 {
                let rate = bits_diff / duration_secs;
                rates.push([current_time, rate]);
            }
        }
    }

    pub fn start_thread(mut self) {
        tokio::spawn(async move {
            let started = Instant::now();
            loop {
                let b = self.canopen_info.lock().await.rx_bits;
                if self.data.len() > RATES_LEN + RATE_PERIOD {
                    self.data.pop_front();
                }

                self.data.push_back((started.elapsed().as_secs_f64(), b));
                self.calculate_rate().await;
                sleep(Duration::from_millis(10)).await;
            }
        });
    }
}
