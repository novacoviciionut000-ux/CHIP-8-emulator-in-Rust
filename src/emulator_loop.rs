use crate::state::Cpu;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const INSTRUCTION_CLOCK_HZ: u32 = 500;
const HARDWARE_CLOCK_HZ: u32 = 60;
const LOOP_SLEEP: Duration = Duration::from_millis(1);

pub struct Clock {
    last_tick: Instant,
    period: Duration,
}

impl Clock {
    pub fn new(hertz: u32) -> Self {
        Self {
            last_tick: Instant::now(),
            period: Duration::from_micros(1_000_000 / hertz as u64),
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.last_tick.elapsed() >= self.period {
            self.last_tick = Instant::now();
            true
        } else {
            false
        }
    }
}

trait Clockable {
    fn execute_tick<F: FnMut(&mut Cpu)>(&mut self, clock: &mut Clock, behaviour: F);
}

impl Clockable for Cpu {
    fn execute_tick<F: FnMut(&mut Cpu)>(&mut self, clock: &mut Clock, mut behaviour: F) {
        if clock.tick() {
            behaviour(self);
        }
    }
}

pub fn behaviour_for_instruction_clock(cpu: &mut Cpu) {
    cpu.fetch_instruction_increment_execute();
}

pub fn emulate_loop(
    shared_cpu: Arc<Mutex<Cpu>>,
    is_running: Arc<AtomicBool>,
    should_beep: Arc<AtomicBool>,
) {
    let mut instruction_clock = Clock::new(INSTRUCTION_CLOCK_HZ);
    let mut hardware_clock = Clock::new(HARDWARE_CLOCK_HZ);

    while is_running.load(Ordering::Relaxed) {
        if let Ok(mut cpu) = shared_cpu.lock() {
            cpu.execute_tick(&mut instruction_clock, behaviour_for_instruction_clock);

            // Decrementing the timer and publishing the beep flag happen
            // in the same tick, under the same lock, on the same thread.
            // There is no separate poll that can miss a one-tick beep.
            cpu.execute_tick(&mut hardware_clock, |cpu| {
                if cpu.delay_timer > 0 {
                    cpu.delay_timer -= 1;
                }
                if cpu.sound_timer > 0 {
                    cpu.sound_timer -= 1;
                }
                should_beep.store(cpu.sound_timer > 0, Ordering::Relaxed);
            });
        }

        thread::sleep(LOOP_SLEEP);
    }

    should_beep.store(false, Ordering::Relaxed);
}