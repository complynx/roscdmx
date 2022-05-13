use std::time::{Duration, Instant};

pub struct Timer {
	tolerance: Duration
}

impl Timer {
	pub fn new() -> Timer {
		unsafe {
			windows::Win32::Media::timeBeginPeriod(1);
		}
		
		let now = Instant::now();
		Timer::coarse_sleep_tick();
		return Timer{
			tolerance: Duration::from_micros((now.elapsed().as_micros() * 3 / 2) as u64),
		};
	}
	fn busy_sleep(&self, till: Instant) {
		while Instant::now().saturating_duration_since(till) == Duration::ZERO {}
	}
	const NS: Duration = Duration::from_nanos(1);
	fn coarse_sleep_tick() {
		std::thread::sleep(Timer::NS);
	}
	pub fn sleep(&mut self, till: Instant) {
		while till.saturating_duration_since(Instant::now()) > self.tolerance {
			Timer::coarse_sleep_tick();
		}
		self.busy_sleep(till);
	}
	pub fn sleep_for(&mut self, duration: Duration) {
		use std::ops::Add;
		self.sleep(Instant::now().add(duration));
	}
}